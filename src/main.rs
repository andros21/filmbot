use log::{error, info, warn};

use std::collections::{hash_map::DefaultHasher, hash_map::RandomState, HashMap};
use std::hash::{BuildHasher, Hash, Hasher};
use teloxide::{
    dispatching::dialogue::{serializer::Json, ErasedStorage, SqliteStorage, Storage},
    prelude::*,
    types::ParseMode::MarkdownV2,
    utils::command::BotCommands,
    utils::markdown::escape,
};

type MyDialogue = Dialogue<State, ErasedStorage<State>>;
type MyStorage = std::sync::Arc<ErasedStorage<State>>;
type MyError = Box<dyn std::error::Error + Send + Sync>;
type HandlerResult = Result<(), MyError>;

struct ByteBuf<'a>(&'a [u8]);

impl<'a> std::fmt::LowerHex for ByteBuf<'a> {
    fn fmt(&self, fmtr: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        for byte in self.0 {
            fmtr.write_fmt(format_args!("{:02x}", byte))?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Hash, Default, serde::Serialize, serde::Deserialize)]
struct Film {
    title: String,
    year: u32,
    url: String,
}

impl Film {
    fn new(title: &str, year: u32, url: &str) -> Self {
        Self {
            title: String::from(title),
            year,
            url: String::from(url),
        }
    }
}

fn sha<T: Hash>(t: &T) -> String {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
        .to_be_bytes()
        .to_vec()
        .iter()
        .take(3)
        .map(|b| format!("{:02x}", b).to_string())
        .collect::<Vec<String>>()
        .join("")
}

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
struct State {
    seen: HashMap<String, Film>,
    tips: HashMap<String, Film>,
}

impl State {
    fn add(&self, id: String, film: Film) -> Self {
        let mut state = self.clone();
        state.tips.insert(id, film);
        state
    }
    fn mv(&self, id: String) -> Option<Self> {
        let mut state = self.clone();
        let film = state.tips.remove(&id)?;
        state.seen.insert(id, film);
        Some(state)
    }
    fn rm(&self, id: String) -> Option<Self> {
        let mut state = self.clone();
        state.seen.remove(&id)?;
        Some(state)
    }
    fn rnd(&self) -> Option<(String, Film)> {
        if !self.tips.is_empty() {
            let state = self.clone();
            let hasher = RandomState::new().build_hasher();
            let rand = hasher.finish() as f64 / std::u64::MAX as f64;
            let urand = (rand * state.tips.len() as f64) as usize;
            let krand = state.tips.keys().cloned().collect::<Vec<String>>()[urand].clone();
            Some((krand.clone(), state.tips.get(&krand).unwrap().clone()))
        } else {
            None
        }
    }
    fn ls(&self) -> String {
        if !self.tips.is_empty() {
            self.tips
                .iter()
                .map(|(id, film)| {
                    format!(
                        "[{}]({})   {}",
                        id,
                        escape(film.url.as_str()),
                        escape(film.title.as_str())
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            String::new()
        }
    }
    fn la(&self) -> String {
        if !self.seen.is_empty() {
            self.seen
                .iter()
                .map(|(id, film)| {
                    format!(
                        "[{}]({})   {}",
                        id,
                        escape(film.url.as_str()),
                        escape(film.title.as_str())
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
        } else {
            String::new()
        }
    }
    fn who(&self, id: String) -> Option<String> {
        if self.tips.contains_key(&id) {
            let film = self.tips.get(&id).unwrap();
            Some(format!("{:#?}", film))
        } else if self.seen.contains_key(&id) {
            let film = self.seen.get(&id).unwrap();
            Some(format!("{:#?}", film))
        } else {
            None
        }
    }
}

#[derive(Clone, BotCommands)]
#[command(rename_rule = "lowercase", description = "🍿 Commands supported:")]
enum Command {
    #[command(description = "Append to tips")]
    Add(String),
    #[command(description = "Move to seen")]
    Mv(String),
    #[command(description = "Weekly challenge")]
    Rnd,
    #[command(description = "Remove from seen")]
    Rm(String),
    #[command(description = "List tips")]
    Ls,
    #[command(description = "List all seen")]
    La,
    #[command(description = "Film info")]
    Who(String),
    #[command(description = "Help message")]
    Help,
}

#[tokio::main]
async fn main() {
    pretty_env_logger::init();
    info!("Starting bot ...");

    let bot = Bot::from_env();

    let storage: MyStorage = SqliteStorage::open("data/db.sqlite", Json)
        .await
        .unwrap()
        .erase();

    let handler = Update::filter_message()
        .enter_dialogue::<Message, ErasedStorage<State>, State>()
        .filter_command::<Command>()
        .endpoint(auth_command);

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![storage])
        .default_handler(|_| async move {})
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

async fn auth_command(bot: Bot, dialogue: MyDialogue, msg: Message, cmd: Command) -> HandlerResult {
    match std::env::var("ALLOWED_ID") {
        Ok(id) => match id.parse::<i64>() {
            Ok(iid) => {
                if msg.chat.id.0 == iid {
                    got_command(bot, dialogue, msg, cmd).await
                } else {
                    warn!("Unauthorized chat id: {:?}", msg);
                    Ok(())
                }
            }
            Err(_) => got_command(bot, dialogue, msg, cmd).await,
        },
        Err(_) => got_command(bot, dialogue, msg, cmd).await,
    }
}

async fn got_command(bot: Bot, dialogue: MyDialogue, msg: Message, cmd: Command) -> HandlerResult {
    match cmd {
        Command::Add(film_str) => {
            if !film_str.is_empty() {
                let items: Vec<&str> = film_str.split(",,,").collect();
                if items.len() != 3 {
                    error!("Invalid items number: {:?}", items);
                    bot.send_message(
                        msg.chat.id,
                        "⚠️ Invalid items number: title,,,year,,,url".to_string(),
                    )
                    .await?;
                } else {
                    let year = items[1].parse::<u32>().unwrap_or_default();
                    if !(1800..=2140).contains(&year) {
                        error!("Invalid film year: {:?}", year);
                        bot.send_message(
                            msg.chat.id,
                            "⚠️ Invalid film year: ∈ [1800, 2140]".to_string(),
                        )
                        .await?;
                    } else {
                        let url = items[2];
                        if url.starts_with("https://") {
                            let film = Film::new(items[0], year, url);
                            let cur_state = dialogue.get_or_default().await?;
                            let state = cur_state.add(sha(&film), film.clone());
                            dialogue.update(state).await?;
                            info!("Film {:?} successfully added to tips", film);
                            bot.send_message(msg.chat.id, "👍 Film sucessfully added!".to_string())
                                .await?;
                        } else {
                            error!("Add failed, invalid film url: {:?}", url);
                            bot.send_message(
                                msg.chat.id,
                                "⚠️ Add failed, invalid film url".to_string(),
                            )
                            .await?;
                        };
                    };
                };
            } else {
                bot.send_message(
                    msg.chat.id,
                    "ℹ️ Usage:   /add <title>,,,<year>,,,<url>".to_string(),
                )
                .await?;
            };
        }
        Command::Mv(id) => {
            if !id.is_empty() {
                let cur_state = dialogue.get_or_default().await?;
                let new_state = cur_state.mv(id.clone());
                match new_state {
                    Some(state) => {
                        dialogue.update(state).await?;
                        info!("Film {:?} successfully moved", id);
                        bot.send_message(msg.chat.id, "👍 Film sucessfully moved".to_string())
                            .await?;
                    }
                    None => {
                        error!("Move failed, invalid film id {:?}", id);
                        bot.send_message(msg.chat.id, "⚠️ Move failed, invalid film id".to_string())
                            .await?;
                    }
                }
            } else {
                bot.send_message(msg.chat.id, "ℹ️ Usage:   /mv <tips film id>".to_string())
                    .await?;
            }
        }
        Command::Rnd => {
            let cur_state = dialogue.get_or_default().await?;
            let rnd_film = cur_state.rnd();
            match rnd_film {
                Some((id, film)) => {
                    bot.send_message(
                        msg.chat.id,
                        format!(
                            "🎲✨ *Weekly challenge*\n\n[{}]({})   {}",
                            id,
                            escape(film.url.as_str()),
                            escape(film.title.as_str())
                        ),
                    )
                    .parse_mode(MarkdownV2)
                    .send()
                    .await?;
                }
                None => {
                    error!("Random challenge failed, is *tips* empty?");
                    bot.send_message(
                        msg.chat.id,
                        "⚠️ Random challenge failed, is *tips* empty?".to_string(),
                    )
                    .parse_mode(MarkdownV2)
                    .send()
                    .await?;
                }
            }
        }
        Command::Rm(id) => {
            if !id.is_empty() {
                let cur_state = dialogue.get_or_default().await?;
                let new_state = cur_state.rm(id.clone());
                match new_state {
                    Some(state) => {
                        dialogue.update(state).await?;
                        info!("Film {} successfully removed", id);
                        bot.send_message(msg.chat.id, "👍 Film sucessfully removed".to_string())
                            .await?;
                    }
                    None => {
                        error!("Remove failed, invalid film {:?}", id);
                        bot.send_message(
                            msg.chat.id,
                            "⚠️ Remove failed, invalid film id".to_string(),
                        )
                        .await?;
                    }
                }
            } else {
                bot.send_message(msg.chat.id, "ℹ️ Usage:   /rm <seen film id>".to_string())
                    .await?;
            }
        }
        Command::Ls => {
            let cur_state = dialogue.get_or_default().await?;
            bot.send_message(msg.chat.id, format!("📌 *Tips*\n\n{}", cur_state.ls()))
                .parse_mode(MarkdownV2)
                .disable_web_page_preview(true)
                .send()
                .await?;
        }
        Command::La => {
            let cur_state = dialogue.get_or_default().await?;
            bot.send_message(msg.chat.id, format!("✅ *Seen*\n\n{}", cur_state.la()))
                .parse_mode(MarkdownV2)
                .disable_web_page_preview(true)
                .send()
                .await?;
        }
        Command::Who(id) => {
            if !id.is_empty() {
                let cur_state = dialogue.get_or_default().await?;
                let info = cur_state.who(id.clone());
                match info {
                    Some(text) => {
                        bot.send_message(
                            msg.chat.id,
                            format!("ℹ️ *Who is {}?*\n\n{}", id, escape(text.as_str())),
                        )
                        .parse_mode(MarkdownV2)
                        .send()
                        .await?;
                    }
                    None => {
                        error!("Info failed, invalid film {:?}", id);
                        bot.send_message(msg.chat.id, "⚠️ Info failed, invalid film id".to_string())
                            .await?;
                    }
                }
            } else {
                bot.send_message(msg.chat.id, "ℹ️ Usage:   /who <film id>".to_string())
                    .await?;
            }
        }
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
    }
    Ok(())
}
