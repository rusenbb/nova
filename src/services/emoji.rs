//! Emoji picker module for searching and inserting emoji

use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// An emoji entry with character and searchable names
#[derive(Debug, Clone)]
pub struct Emoji {
    pub char: &'static str,
    pub names: &'static [&'static str],
}

impl Emoji {
    /// Get the primary name
    pub fn name(&self) -> &str {
        self.names.first().unwrap_or(&"emoji")
    }

    /// Get all names as comma-separated string
    pub fn aliases(&self) -> String {
        self.names.join(", ")
    }
}

/// Search for emojis matching a query
pub fn search(query: &str, max_results: usize) -> Vec<&'static Emoji> {
    let query = query.trim().to_lowercase();

    if query.is_empty() {
        // Return popular emojis when no query
        return EMOJIS.iter().take(max_results).collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(i64, &'static Emoji)> = Vec::new();

    for emoji in EMOJIS.iter() {
        // Check all names for matches
        let mut best_score: Option<i64> = None;
        for name in emoji.names {
            if let Some(score) = matcher.fuzzy_match(name, &query) {
                best_score = Some(best_score.map_or(score, |s| s.max(score)));
            }
        }

        // Boost exact prefix matches
        if let Some(score) = best_score {
            let boosted = if emoji.names.iter().any(|n| n.starts_with(&query)) {
                score + 100
            } else {
                score
            };
            results.push((boosted, emoji));
        }
    }

    // Sort by score descending
    results.sort_by(|a, b| b.0.cmp(&a.0));
    results
        .into_iter()
        .take(max_results)
        .map(|(_, emoji)| emoji)
        .collect()
}

/// Common emojis with searchable names
static EMOJIS: &[Emoji] = &[
    // Smileys & Emotion
    Emoji {
        char: "😀",
        names: &["grinning", "smile", "happy"],
    },
    Emoji {
        char: "😃",
        names: &["smiley", "happy", "joy"],
    },
    Emoji {
        char: "😄",
        names: &["smile", "happy", "joy"],
    },
    Emoji {
        char: "😁",
        names: &["grin", "happy"],
    },
    Emoji {
        char: "😅",
        names: &["sweat_smile", "nervous"],
    },
    Emoji {
        char: "😂",
        names: &["joy", "laugh", "crying", "tears"],
    },
    Emoji {
        char: "🤣",
        names: &["rofl", "laughing", "rolling"],
    },
    Emoji {
        char: "😊",
        names: &["blush", "smile", "happy"],
    },
    Emoji {
        char: "😇",
        names: &["innocent", "angel", "halo"],
    },
    Emoji {
        char: "🙂",
        names: &["slight_smile", "smile"],
    },
    Emoji {
        char: "😉",
        names: &["wink", "flirt"],
    },
    Emoji {
        char: "😌",
        names: &["relieved", "calm"],
    },
    Emoji {
        char: "😍",
        names: &["heart_eyes", "love", "crush"],
    },
    Emoji {
        char: "🥰",
        names: &["smiling_hearts", "love", "adore"],
    },
    Emoji {
        char: "😘",
        names: &["kiss", "blow_kiss", "love"],
    },
    Emoji {
        char: "😋",
        names: &["yum", "delicious", "tasty"],
    },
    Emoji {
        char: "😎",
        names: &["sunglasses", "cool"],
    },
    Emoji {
        char: "🤓",
        names: &["nerd", "geek", "glasses"],
    },
    Emoji {
        char: "🧐",
        names: &["monocle", "thinking", "curious"],
    },
    Emoji {
        char: "🤔",
        names: &["thinking", "hmm", "wonder"],
    },
    Emoji {
        char: "🤨",
        names: &["raised_eyebrow", "skeptic", "sus"],
    },
    Emoji {
        char: "😐",
        names: &["neutral", "meh", "blank"],
    },
    Emoji {
        char: "😑",
        names: &["expressionless", "blank"],
    },
    Emoji {
        char: "😶",
        names: &["no_mouth", "silent", "speechless"],
    },
    Emoji {
        char: "😏",
        names: &["smirk", "smug"],
    },
    Emoji {
        char: "😒",
        names: &["unamused", "meh", "bored"],
    },
    Emoji {
        char: "🙄",
        names: &["eye_roll", "whatever"],
    },
    Emoji {
        char: "😬",
        names: &["grimace", "awkward", "cringe"],
    },
    Emoji {
        char: "😮‍💨",
        names: &["exhale", "sigh", "relief"],
    },
    Emoji {
        char: "🤥",
        names: &["lying", "pinocchio"],
    },
    Emoji {
        char: "😌",
        names: &["relieved", "peaceful"],
    },
    Emoji {
        char: "😔",
        names: &["pensive", "sad", "thoughtful"],
    },
    Emoji {
        char: "😪",
        names: &["sleepy", "tired"],
    },
    Emoji {
        char: "🤤",
        names: &["drool", "drooling"],
    },
    Emoji {
        char: "😴",
        names: &["sleeping", "zzz", "tired"],
    },
    Emoji {
        char: "😷",
        names: &["mask", "sick", "covid"],
    },
    Emoji {
        char: "🤒",
        names: &["thermometer", "sick", "fever"],
    },
    Emoji {
        char: "🤕",
        names: &["bandage", "hurt", "injured"],
    },
    Emoji {
        char: "🤢",
        names: &["nauseated", "sick", "green"],
    },
    Emoji {
        char: "🤮",
        names: &["vomit", "puke", "sick"],
    },
    Emoji {
        char: "🤧",
        names: &["sneeze", "sick", "achoo"],
    },
    Emoji {
        char: "🥵",
        names: &["hot", "sweating", "heat"],
    },
    Emoji {
        char: "🥶",
        names: &["cold", "freezing", "frozen"],
    },
    Emoji {
        char: "🥴",
        names: &["woozy", "drunk", "dizzy"],
    },
    Emoji {
        char: "😵",
        names: &["dizzy", "dead", "knocked_out"],
    },
    Emoji {
        char: "🤯",
        names: &["exploding_head", "mind_blown", "shocked"],
    },
    Emoji {
        char: "🤠",
        names: &["cowboy", "yeehaw"],
    },
    Emoji {
        char: "🥳",
        names: &["party", "celebration", "birthday"],
    },
    Emoji {
        char: "🥸",
        names: &["disguise", "incognito", "glasses"],
    },
    Emoji {
        char: "😎",
        names: &["cool", "sunglasses", "awesome"],
    },
    Emoji {
        char: "😕",
        names: &["confused", "puzzled"],
    },
    Emoji {
        char: "😟",
        names: &["worried", "concerned"],
    },
    Emoji {
        char: "🙁",
        names: &["frown", "sad"],
    },
    Emoji {
        char: "😮",
        names: &["open_mouth", "surprised", "wow"],
    },
    Emoji {
        char: "😯",
        names: &["hushed", "surprised"],
    },
    Emoji {
        char: "😲",
        names: &["astonished", "shocked", "wow"],
    },
    Emoji {
        char: "😳",
        names: &["flushed", "embarrassed", "shocked"],
    },
    Emoji {
        char: "🥺",
        names: &["pleading", "puppy_eyes", "please"],
    },
    Emoji {
        char: "😦",
        names: &["frowning", "sad"],
    },
    Emoji {
        char: "😧",
        names: &["anguished", "worried"],
    },
    Emoji {
        char: "😨",
        names: &["fearful", "scared", "afraid"],
    },
    Emoji {
        char: "😰",
        names: &["anxious", "nervous", "sweat"],
    },
    Emoji {
        char: "😥",
        names: &["sad", "disappointed", "relieved"],
    },
    Emoji {
        char: "😢",
        names: &["cry", "sad", "tear"],
    },
    Emoji {
        char: "😭",
        names: &["sob", "crying", "sad", "tears"],
    },
    Emoji {
        char: "😱",
        names: &["scream", "scared", "horror"],
    },
    Emoji {
        char: "😖",
        names: &["confounded", "frustrated"],
    },
    Emoji {
        char: "😣",
        names: &["persevere", "struggle"],
    },
    Emoji {
        char: "😞",
        names: &["disappointed", "sad"],
    },
    Emoji {
        char: "😓",
        names: &["sweat", "nervous", "anxious"],
    },
    Emoji {
        char: "😩",
        names: &["weary", "tired", "exhausted"],
    },
    Emoji {
        char: "😫",
        names: &["tired", "exhausted"],
    },
    Emoji {
        char: "🥱",
        names: &["yawn", "tired", "sleepy", "bored"],
    },
    Emoji {
        char: "😤",
        names: &["triumph", "proud", "huffing"],
    },
    Emoji {
        char: "😡",
        names: &["rage", "angry", "mad"],
    },
    Emoji {
        char: "😠",
        names: &["angry", "mad", "grumpy"],
    },
    Emoji {
        char: "🤬",
        names: &["cursing", "swearing", "angry"],
    },
    Emoji {
        char: "😈",
        names: &["smiling_imp", "devil", "evil"],
    },
    Emoji {
        char: "👿",
        names: &["imp", "devil", "angry"],
    },
    Emoji {
        char: "💀",
        names: &["skull", "dead", "death"],
    },
    Emoji {
        char: "☠️",
        names: &["skull_crossbones", "danger", "death"],
    },
    Emoji {
        char: "💩",
        names: &["poop", "poo", "shit"],
    },
    Emoji {
        char: "🤡",
        names: &["clown", "joker"],
    },
    Emoji {
        char: "👹",
        names: &["ogre", "monster", "demon"],
    },
    Emoji {
        char: "👺",
        names: &["goblin", "tengu", "monster"],
    },
    Emoji {
        char: "👻",
        names: &["ghost", "boo", "spooky"],
    },
    Emoji {
        char: "👽",
        names: &["alien", "ufo", "extraterrestrial"],
    },
    Emoji {
        char: "👾",
        names: &["space_invader", "alien", "game"],
    },
    Emoji {
        char: "🤖",
        names: &["robot", "bot", "android"],
    },
    // Gestures & Body
    Emoji {
        char: "👋",
        names: &["wave", "hello", "bye", "hi"],
    },
    Emoji {
        char: "🤚",
        names: &["raised_back_hand", "stop"],
    },
    Emoji {
        char: "🖐️",
        names: &["hand", "high_five", "stop"],
    },
    Emoji {
        char: "✋",
        names: &["raised_hand", "stop", "high_five"],
    },
    Emoji {
        char: "🖖",
        names: &["vulcan", "spock", "star_trek"],
    },
    Emoji {
        char: "👌",
        names: &["ok", "okay", "perfect"],
    },
    Emoji {
        char: "🤌",
        names: &["pinched_fingers", "italian", "chef"],
    },
    Emoji {
        char: "🤏",
        names: &["pinching", "small", "tiny"],
    },
    Emoji {
        char: "✌️",
        names: &["peace", "victory", "v"],
    },
    Emoji {
        char: "🤞",
        names: &["crossed_fingers", "luck", "hope"],
    },
    Emoji {
        char: "🤟",
        names: &["love_you", "rock", "ily"],
    },
    Emoji {
        char: "🤘",
        names: &["rock", "metal", "horns"],
    },
    Emoji {
        char: "🤙",
        names: &["call_me", "shaka", "hang_loose"],
    },
    Emoji {
        char: "👈",
        names: &["point_left", "left"],
    },
    Emoji {
        char: "👉",
        names: &["point_right", "right"],
    },
    Emoji {
        char: "👆",
        names: &["point_up", "up"],
    },
    Emoji {
        char: "🖕",
        names: &["middle_finger", "fu", "fuck"],
    },
    Emoji {
        char: "👇",
        names: &["point_down", "down"],
    },
    Emoji {
        char: "☝️",
        names: &["point_up", "one", "wait"],
    },
    Emoji {
        char: "👍",
        names: &["thumbsup", "yes", "good", "like", "+1"],
    },
    Emoji {
        char: "👎",
        names: &["thumbsdown", "no", "bad", "dislike", "-1"],
    },
    Emoji {
        char: "✊",
        names: &["fist", "punch", "power"],
    },
    Emoji {
        char: "👊",
        names: &["punch", "fist_bump"],
    },
    Emoji {
        char: "🤛",
        names: &["left_fist", "fist_bump"],
    },
    Emoji {
        char: "🤜",
        names: &["right_fist", "fist_bump"],
    },
    Emoji {
        char: "👏",
        names: &["clap", "applause", "bravo"],
    },
    Emoji {
        char: "🙌",
        names: &["raised_hands", "hooray", "yay"],
    },
    Emoji {
        char: "👐",
        names: &["open_hands", "hug"],
    },
    Emoji {
        char: "🤲",
        names: &["palms_up", "cupped_hands"],
    },
    Emoji {
        char: "🤝",
        names: &["handshake", "deal", "agreement"],
    },
    Emoji {
        char: "🙏",
        names: &["pray", "please", "thanks", "namaste"],
    },
    Emoji {
        char: "✍️",
        names: &["writing", "write"],
    },
    Emoji {
        char: "💪",
        names: &["muscle", "strong", "flex", "bicep"],
    },
    // Hearts & Love
    Emoji {
        char: "❤️",
        names: &["heart", "love", "red_heart"],
    },
    Emoji {
        char: "🧡",
        names: &["orange_heart", "heart"],
    },
    Emoji {
        char: "💛",
        names: &["yellow_heart", "heart"],
    },
    Emoji {
        char: "💚",
        names: &["green_heart", "heart"],
    },
    Emoji {
        char: "💙",
        names: &["blue_heart", "heart"],
    },
    Emoji {
        char: "💜",
        names: &["purple_heart", "heart"],
    },
    Emoji {
        char: "🖤",
        names: &["black_heart", "heart"],
    },
    Emoji {
        char: "🤍",
        names: &["white_heart", "heart"],
    },
    Emoji {
        char: "🤎",
        names: &["brown_heart", "heart"],
    },
    Emoji {
        char: "💔",
        names: &["broken_heart", "heartbreak", "sad"],
    },
    Emoji {
        char: "💕",
        names: &["two_hearts", "love"],
    },
    Emoji {
        char: "💞",
        names: &["revolving_hearts", "love"],
    },
    Emoji {
        char: "💓",
        names: &["heartbeat", "love"],
    },
    Emoji {
        char: "💗",
        names: &["growing_heart", "love"],
    },
    Emoji {
        char: "💖",
        names: &["sparkling_heart", "love"],
    },
    Emoji {
        char: "💘",
        names: &["cupid", "love", "arrow"],
    },
    Emoji {
        char: "💝",
        names: &["gift_heart", "love", "present"],
    },
    // Objects & Symbols
    Emoji {
        char: "🔥",
        names: &["fire", "hot", "lit", "flame"],
    },
    Emoji {
        char: "✨",
        names: &["sparkles", "stars", "magic", "new"],
    },
    Emoji {
        char: "⭐",
        names: &["star", "favorite"],
    },
    Emoji {
        char: "🌟",
        names: &["glowing_star", "star", "shine"],
    },
    Emoji {
        char: "💫",
        names: &["dizzy", "star", "shooting"],
    },
    Emoji {
        char: "💥",
        names: &["boom", "explosion", "collision"],
    },
    Emoji {
        char: "💢",
        names: &["anger", "angry", "vein"],
    },
    Emoji {
        char: "💦",
        names: &["sweat_drops", "water", "splash"],
    },
    Emoji {
        char: "💨",
        names: &["dash", "wind", "fast", "running"],
    },
    Emoji {
        char: "🕳️",
        names: &["hole", "black_hole"],
    },
    Emoji {
        char: "💣",
        names: &["bomb", "explosive"],
    },
    Emoji {
        char: "💬",
        names: &["speech_bubble", "chat", "comment"],
    },
    Emoji {
        char: "👁️‍🗨️",
        names: &["eye_bubble", "witness"],
    },
    Emoji {
        char: "🗨️",
        names: &["left_speech", "bubble"],
    },
    Emoji {
        char: "🗯️",
        names: &["right_anger", "bubble"],
    },
    Emoji {
        char: "💭",
        names: &["thought_bubble", "thinking"],
    },
    Emoji {
        char: "💤",
        names: &["zzz", "sleep", "tired"],
    },
    Emoji {
        char: "👀",
        names: &["eyes", "look", "see", "watching"],
    },
    Emoji {
        char: "👁️",
        names: &["eye", "see"],
    },
    Emoji {
        char: "👂",
        names: &["ear", "hear", "listen"],
    },
    Emoji {
        char: "👃",
        names: &["nose", "smell"],
    },
    Emoji {
        char: "👅",
        names: &["tongue", "lick", "taste"],
    },
    Emoji {
        char: "👄",
        names: &["lips", "mouth", "kiss"],
    },
    // Tech & Work
    Emoji {
        char: "💻",
        names: &["laptop", "computer", "mac"],
    },
    Emoji {
        char: "🖥️",
        names: &["desktop", "computer", "pc"],
    },
    Emoji {
        char: "⌨️",
        names: &["keyboard", "type"],
    },
    Emoji {
        char: "🖱️",
        names: &["mouse", "click"],
    },
    Emoji {
        char: "📱",
        names: &["phone", "iphone", "mobile", "smartphone"],
    },
    Emoji {
        char: "📧",
        names: &["email", "mail", "envelope"],
    },
    Emoji {
        char: "📝",
        names: &["memo", "note", "write"],
    },
    Emoji {
        char: "📎",
        names: &["paperclip", "attachment"],
    },
    Emoji {
        char: "📌",
        names: &["pushpin", "pin"],
    },
    Emoji {
        char: "📍",
        names: &["pin", "location", "map"],
    },
    Emoji {
        char: "🔗",
        names: &["link", "chain", "url"],
    },
    Emoji {
        char: "🔒",
        names: &["lock", "locked", "secure"],
    },
    Emoji {
        char: "🔓",
        names: &["unlock", "unlocked", "open"],
    },
    Emoji {
        char: "🔑",
        names: &["key", "password"],
    },
    Emoji {
        char: "🔧",
        names: &["wrench", "tool", "fix"],
    },
    Emoji {
        char: "🔨",
        names: &["hammer", "tool", "build"],
    },
    Emoji {
        char: "⚙️",
        names: &["gear", "settings", "cog"],
    },
    Emoji {
        char: "🛠️",
        names: &["tools", "build", "fix"],
    },
    Emoji {
        char: "📦",
        names: &["package", "box", "shipping"],
    },
    Emoji {
        char: "🗑️",
        names: &["trash", "delete", "garbage"],
    },
    Emoji {
        char: "📁",
        names: &["folder", "directory"],
    },
    Emoji {
        char: "📂",
        names: &["open_folder", "directory"],
    },
    Emoji {
        char: "📄",
        names: &["document", "file", "page"],
    },
    Emoji {
        char: "📊",
        names: &["chart", "graph", "stats"],
    },
    Emoji {
        char: "📈",
        names: &["chart_up", "trending", "growth"],
    },
    Emoji {
        char: "📉",
        names: &["chart_down", "decline", "loss"],
    },
    Emoji {
        char: "✅",
        names: &["check", "done", "yes", "complete"],
    },
    Emoji {
        char: "❌",
        names: &["x", "no", "wrong", "cross", "cancel"],
    },
    Emoji {
        char: "❓",
        names: &["question", "what", "help"],
    },
    Emoji {
        char: "❗",
        names: &["exclamation", "important", "alert"],
    },
    Emoji {
        char: "⚠️",
        names: &["warning", "caution", "alert"],
    },
    Emoji {
        char: "🚀",
        names: &["rocket", "launch", "ship", "fast"],
    },
    Emoji {
        char: "🎉",
        names: &["party", "tada", "celebration", "congrats"],
    },
    Emoji {
        char: "🎊",
        names: &["confetti", "party", "celebration"],
    },
    Emoji {
        char: "🎁",
        names: &["gift", "present", "birthday"],
    },
    Emoji {
        char: "🏆",
        names: &["trophy", "winner", "award", "champion"],
    },
    Emoji {
        char: "🥇",
        names: &["gold_medal", "first", "winner"],
    },
    Emoji {
        char: "🥈",
        names: &["silver_medal", "second"],
    },
    Emoji {
        char: "🥉",
        names: &["bronze_medal", "third"],
    },
    Emoji {
        char: "⏰",
        names: &["alarm", "clock", "time"],
    },
    Emoji {
        char: "⏱️",
        names: &["stopwatch", "timer"],
    },
    Emoji {
        char: "⌛",
        names: &["hourglass", "time", "wait"],
    },
    Emoji {
        char: "⏳",
        names: &["hourglass_flowing", "time", "loading"],
    },
    // Weather & Nature
    Emoji {
        char: "☀️",
        names: &["sun", "sunny", "weather"],
    },
    Emoji {
        char: "🌤️",
        names: &["partly_sunny", "weather"],
    },
    Emoji {
        char: "⛅",
        names: &["partly_cloudy", "weather"],
    },
    Emoji {
        char: "🌥️",
        names: &["mostly_cloudy", "weather"],
    },
    Emoji {
        char: "☁️",
        names: &["cloud", "cloudy", "weather"],
    },
    Emoji {
        char: "🌧️",
        names: &["rain", "rainy", "weather"],
    },
    Emoji {
        char: "⛈️",
        names: &["thunder", "storm", "weather"],
    },
    Emoji {
        char: "🌩️",
        names: &["lightning", "storm", "weather"],
    },
    Emoji {
        char: "❄️",
        names: &["snow", "snowflake", "cold", "winter"],
    },
    Emoji {
        char: "🌈",
        names: &["rainbow", "pride"],
    },
    Emoji {
        char: "🌊",
        names: &["wave", "ocean", "water", "sea"],
    },
    // Food & Drink
    Emoji {
        char: "☕",
        names: &["coffee", "cafe", "hot"],
    },
    Emoji {
        char: "🍵",
        names: &["tea", "green_tea"],
    },
    Emoji {
        char: "🍺",
        names: &["beer", "drink", "alcohol"],
    },
    Emoji {
        char: "🍻",
        names: &["beers", "cheers", "drink"],
    },
    Emoji {
        char: "🍷",
        names: &["wine", "drink", "alcohol"],
    },
    Emoji {
        char: "🍸",
        names: &["cocktail", "martini", "drink"],
    },
    Emoji {
        char: "🍕",
        names: &["pizza", "food"],
    },
    Emoji {
        char: "🍔",
        names: &["burger", "hamburger", "food"],
    },
    Emoji {
        char: "🍟",
        names: &["fries", "french_fries", "food"],
    },
    Emoji {
        char: "🌮",
        names: &["taco", "food", "mexican"],
    },
    Emoji {
        char: "🍜",
        names: &["ramen", "noodles", "soup", "food"],
    },
    Emoji {
        char: "🍣",
        names: &["sushi", "food", "japanese"],
    },
    Emoji {
        char: "🍦",
        names: &["ice_cream", "dessert"],
    },
    Emoji {
        char: "🍰",
        names: &["cake", "dessert", "birthday"],
    },
    Emoji {
        char: "🎂",
        names: &["birthday_cake", "cake", "party"],
    },
    Emoji {
        char: "🍪",
        names: &["cookie", "dessert", "snack"],
    },
    // Animals
    Emoji {
        char: "🐶",
        names: &["dog", "puppy", "pet"],
    },
    Emoji {
        char: "🐱",
        names: &["cat", "kitten", "pet"],
    },
    Emoji {
        char: "🐭",
        names: &["mouse", "rat"],
    },
    Emoji {
        char: "🐰",
        names: &["rabbit", "bunny"],
    },
    Emoji {
        char: "🦊",
        names: &["fox", "animal"],
    },
    Emoji {
        char: "🐻",
        names: &["bear", "animal"],
    },
    Emoji {
        char: "🐼",
        names: &["panda", "bear", "animal"],
    },
    Emoji {
        char: "🐨",
        names: &["koala", "animal"],
    },
    Emoji {
        char: "🐯",
        names: &["tiger", "animal"],
    },
    Emoji {
        char: "🦁",
        names: &["lion", "animal", "king"],
    },
    Emoji {
        char: "🐮",
        names: &["cow", "animal"],
    },
    Emoji {
        char: "🐷",
        names: &["pig", "animal"],
    },
    Emoji {
        char: "🐸",
        names: &["frog", "animal"],
    },
    Emoji {
        char: "🐵",
        names: &["monkey", "animal"],
    },
    Emoji {
        char: "🙈",
        names: &["see_no_evil", "monkey"],
    },
    Emoji {
        char: "🙉",
        names: &["hear_no_evil", "monkey"],
    },
    Emoji {
        char: "🙊",
        names: &["speak_no_evil", "monkey"],
    },
    Emoji {
        char: "🐔",
        names: &["chicken", "animal"],
    },
    Emoji {
        char: "🐧",
        names: &["penguin", "animal"],
    },
    Emoji {
        char: "🐦",
        names: &["bird", "animal"],
    },
    Emoji {
        char: "🦆",
        names: &["duck", "animal"],
    },
    Emoji {
        char: "🦅",
        names: &["eagle", "bird", "america"],
    },
    Emoji {
        char: "🦉",
        names: &["owl", "bird", "night"],
    },
    Emoji {
        char: "🐝",
        names: &["bee", "honey", "insect"],
    },
    Emoji {
        char: "🐛",
        names: &["bug", "insect", "caterpillar"],
    },
    Emoji {
        char: "🦋",
        names: &["butterfly", "insect"],
    },
    Emoji {
        char: "🐌",
        names: &["snail", "slow"],
    },
    Emoji {
        char: "🐢",
        names: &["turtle", "slow", "animal"],
    },
    Emoji {
        char: "🐍",
        names: &["snake", "python", "animal"],
    },
    Emoji {
        char: "🦎",
        names: &["lizard", "reptile"],
    },
    Emoji {
        char: "🦖",
        names: &["dinosaur", "trex", "dino"],
    },
    Emoji {
        char: "🐙",
        names: &["octopus", "sea", "animal"],
    },
    Emoji {
        char: "🦀",
        names: &["crab", "sea", "animal"],
    },
    Emoji {
        char: "🦑",
        names: &["squid", "sea", "animal"],
    },
    Emoji {
        char: "🦐",
        names: &["shrimp", "sea", "prawn"],
    },
    Emoji {
        char: "🐠",
        names: &["fish", "sea", "animal"],
    },
    Emoji {
        char: "🐬",
        names: &["dolphin", "sea", "animal"],
    },
    Emoji {
        char: "🐳",
        names: &["whale", "sea", "animal"],
    },
    Emoji {
        char: "🦈",
        names: &["shark", "sea", "jaws"],
    },
    Emoji {
        char: "🐊",
        names: &["crocodile", "alligator", "animal"],
    },
    // Arrows & Symbols
    Emoji {
        char: "⬆️",
        names: &["arrow_up", "up"],
    },
    Emoji {
        char: "⬇️",
        names: &["arrow_down", "down"],
    },
    Emoji {
        char: "⬅️",
        names: &["arrow_left", "left"],
    },
    Emoji {
        char: "➡️",
        names: &["arrow_right", "right"],
    },
    Emoji {
        char: "↩️",
        names: &["arrow_return", "back", "undo"],
    },
    Emoji {
        char: "↪️",
        names: &["arrow_forward", "redo"],
    },
    Emoji {
        char: "🔄",
        names: &["refresh", "reload", "sync", "arrows"],
    },
    Emoji {
        char: "🔃",
        names: &["clockwise", "arrows"],
    },
    Emoji {
        char: "➕",
        names: &["plus", "add"],
    },
    Emoji {
        char: "➖",
        names: &["minus", "subtract"],
    },
    Emoji {
        char: "✖️",
        names: &["multiply", "x"],
    },
    Emoji {
        char: "➗",
        names: &["divide", "division"],
    },
    Emoji {
        char: "♾️",
        names: &["infinity", "forever"],
    },
    Emoji {
        char: "💲",
        names: &["dollar", "money"],
    },
    Emoji {
        char: "™️",
        names: &["trademark", "tm"],
    },
    Emoji {
        char: "©️",
        names: &["copyright", "c"],
    },
    Emoji {
        char: "®️",
        names: &["registered", "r"],
    },
    Emoji {
        char: "〰️",
        names: &["wavy_dash", "squiggle"],
    },
    Emoji {
        char: "➰",
        names: &["curly_loop", "loop"],
    },
    Emoji {
        char: "〽️",
        names: &["part_alternation", "m"],
    },
    Emoji {
        char: "✳️",
        names: &["asterisk", "star"],
    },
    Emoji {
        char: "✴️",
        names: &["star", "sparkle"],
    },
    Emoji {
        char: "❇️",
        names: &["sparkle", "star"],
    },
    Emoji {
        char: "‼️",
        names: &["bangbang", "exclamation"],
    },
    Emoji {
        char: "⁉️",
        names: &["interrobang", "what"],
    },
    Emoji {
        char: "🔴",
        names: &["red_circle", "circle"],
    },
    Emoji {
        char: "🟠",
        names: &["orange_circle", "circle"],
    },
    Emoji {
        char: "🟡",
        names: &["yellow_circle", "circle"],
    },
    Emoji {
        char: "🟢",
        names: &["green_circle", "circle"],
    },
    Emoji {
        char: "🔵",
        names: &["blue_circle", "circle"],
    },
    Emoji {
        char: "🟣",
        names: &["purple_circle", "circle"],
    },
    Emoji {
        char: "⚫",
        names: &["black_circle", "circle"],
    },
    Emoji {
        char: "⚪",
        names: &["white_circle", "circle"],
    },
    Emoji {
        char: "🟤",
        names: &["brown_circle", "circle"],
    },
    Emoji {
        char: "🔶",
        names: &["orange_diamond", "diamond"],
    },
    Emoji {
        char: "🔷",
        names: &["blue_diamond", "diamond"],
    },
    Emoji {
        char: "🔸",
        names: &["small_orange_diamond", "diamond"],
    },
    Emoji {
        char: "🔹",
        names: &["small_blue_diamond", "diamond"],
    },
    // Misc popular
    Emoji {
        char: "💯",
        names: &["100", "hundred", "perfect", "score"],
    },
    Emoji {
        char: "🆗",
        names: &["ok", "okay"],
    },
    Emoji {
        char: "🆕",
        names: &["new"],
    },
    Emoji {
        char: "🆒",
        names: &["cool"],
    },
    Emoji {
        char: "🆓",
        names: &["free"],
    },
    Emoji {
        char: "🆙",
        names: &["up"],
    },
    Emoji {
        char: "🔝",
        names: &["top"],
    },
    Emoji {
        char: "🔜",
        names: &["soon"],
    },
    Emoji {
        char: "🔛",
        names: &["on"],
    },
    Emoji {
        char: "🔚",
        names: &["end"],
    },
    Emoji {
        char: "🔙",
        names: &["back"],
    },
    Emoji {
        char: "ℹ️",
        names: &["info", "information"],
    },
    Emoji {
        char: "Ⓜ️",
        names: &["m", "metro"],
    },
    Emoji {
        char: "🅿️",
        names: &["p", "parking"],
    },
    Emoji {
        char: "🈁",
        names: &["koko", "japanese"],
    },
    Emoji {
        char: "🔞",
        names: &["no_one_under_18", "adult", "nsfw"],
    },
    Emoji {
        char: "📵",
        names: &["no_mobile", "no_phone"],
    },
    Emoji {
        char: "🔇",
        names: &["mute", "no_sound", "silent"],
    },
    Emoji {
        char: "🔕",
        names: &["no_bell", "silent"],
    },
    Emoji {
        char: "🚫",
        names: &["no_entry", "prohibited", "forbidden"],
    },
    Emoji {
        char: "⛔",
        names: &["no_entry_sign", "stop"],
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_smile() {
        let results = search("smile", 5);
        assert!(!results.is_empty());
        // Should find smiling emojis
        assert!(results
            .iter()
            .any(|e| e.names.iter().any(|n| n.contains("smile"))));
    }

    #[test]
    fn test_search_heart() {
        let results = search("heart", 10);
        assert!(!results.is_empty());
        // Should find heart emojis
        assert!(results
            .iter()
            .any(|e| e.char == "❤️" || e.char.contains('💜')));
    }

    #[test]
    fn test_empty_query() {
        let results = search("", 5);
        assert_eq!(results.len(), 5);
    }
}
