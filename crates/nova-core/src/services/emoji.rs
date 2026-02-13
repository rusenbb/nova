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
        return EMOJIS.iter().take(max_results).collect();
    }

    let matcher = SkimMatcherV2::default();
    let mut results: Vec<(i64, &'static Emoji)> = Vec::new();

    for emoji in EMOJIS.iter() {
        let mut best_score: Option<i64> = None;
        for name in emoji.names {
            if let Some(score) = matcher.fuzzy_match(name, &query) {
                best_score = Some(best_score.map_or(score, |s| s.max(score)));
            }
        }

        if let Some(score) = best_score {
            let boosted = if emoji.names.iter().any(|n| n.starts_with(&query)) {
                score + 100
            } else {
                score
            };
            results.push((boosted, emoji));
        }
    }

    results.sort_by(|a, b| b.0.cmp(&a.0));
    results
        .into_iter()
        .take(max_results)
        .map(|(_, emoji)| emoji)
        .collect()
}

/// Common emojis with searchable names
static EMOJIS: &[Emoji] = &[
    Emoji { char: "\u{1f600}", names: &["grinning", "smile", "happy"] },
    Emoji { char: "\u{1f603}", names: &["smiley", "happy", "joy"] },
    Emoji { char: "\u{1f604}", names: &["smile", "happy", "joy"] },
    Emoji { char: "\u{1f601}", names: &["grin", "happy"] },
    Emoji { char: "\u{1f605}", names: &["sweat_smile", "nervous"] },
    Emoji { char: "\u{1f602}", names: &["joy", "laugh", "crying", "tears"] },
    Emoji { char: "\u{1f923}", names: &["rofl", "laughing", "rolling"] },
    Emoji { char: "\u{1f60a}", names: &["blush", "smile", "happy"] },
    Emoji { char: "\u{1f607}", names: &["innocent", "angel", "halo"] },
    Emoji { char: "\u{1f642}", names: &["slight_smile", "smile"] },
    Emoji { char: "\u{1f609}", names: &["wink", "flirt"] },
    Emoji { char: "\u{1f60c}", names: &["relieved", "calm"] },
    Emoji { char: "\u{1f60d}", names: &["heart_eyes", "love", "crush"] },
    Emoji { char: "\u{1f970}", names: &["smiling_hearts", "love", "adore"] },
    Emoji { char: "\u{1f618}", names: &["kiss", "blow_kiss", "love"] },
    Emoji { char: "\u{1f60b}", names: &["yum", "delicious", "tasty"] },
    Emoji { char: "\u{1f60e}", names: &["sunglasses", "cool"] },
    Emoji { char: "\u{1f913}", names: &["nerd", "geek", "glasses"] },
    Emoji { char: "\u{1f9d0}", names: &["monocle", "thinking", "curious"] },
    Emoji { char: "\u{1f914}", names: &["thinking", "hmm", "wonder"] },
    Emoji { char: "\u{1f928}", names: &["raised_eyebrow", "skeptic", "sus"] },
    Emoji { char: "\u{1f610}", names: &["neutral", "meh", "blank"] },
    Emoji { char: "\u{1f611}", names: &["expressionless", "blank"] },
    Emoji { char: "\u{1f636}", names: &["no_mouth", "silent", "speechless"] },
    Emoji { char: "\u{1f60f}", names: &["smirk", "smug"] },
    Emoji { char: "\u{1f612}", names: &["unamused", "meh", "bored"] },
    Emoji { char: "\u{1f644}", names: &["eye_roll", "whatever"] },
    Emoji { char: "\u{1f62c}", names: &["grimace", "awkward", "cringe"] },
    Emoji { char: "\u{1f62e}\u{200d}\u{1f4a8}", names: &["exhale", "sigh", "relief"] },
    Emoji { char: "\u{1f925}", names: &["lying", "pinocchio"] },
    Emoji { char: "\u{1f614}", names: &["pensive", "sad", "thoughtful"] },
    Emoji { char: "\u{1f62a}", names: &["sleepy", "tired"] },
    Emoji { char: "\u{1f924}", names: &["drool", "drooling"] },
    Emoji { char: "\u{1f634}", names: &["sleeping", "zzz", "tired"] },
    Emoji { char: "\u{1f637}", names: &["mask", "sick", "covid"] },
    Emoji { char: "\u{1f912}", names: &["thermometer", "sick", "fever"] },
    Emoji { char: "\u{1f915}", names: &["bandage", "hurt", "injured"] },
    Emoji { char: "\u{1f922}", names: &["nauseated", "sick", "green"] },
    Emoji { char: "\u{1f92e}", names: &["vomit", "puke", "sick"] },
    Emoji { char: "\u{1f927}", names: &["sneeze", "sick", "achoo"] },
    Emoji { char: "\u{1f975}", names: &["hot", "sweating", "heat"] },
    Emoji { char: "\u{1f976}", names: &["cold", "freezing", "frozen"] },
    Emoji { char: "\u{1f974}", names: &["woozy", "drunk", "dizzy"] },
    Emoji { char: "\u{1f635}", names: &["dizzy", "dead", "knocked_out"] },
    Emoji { char: "\u{1f92f}", names: &["exploding_head", "mind_blown", "shocked"] },
    Emoji { char: "\u{1f920}", names: &["cowboy", "yeehaw"] },
    Emoji { char: "\u{1f973}", names: &["party", "celebration", "birthday"] },
    Emoji { char: "\u{1f978}", names: &["disguise", "incognito", "glasses"] },
    Emoji { char: "\u{1f615}", names: &["confused", "puzzled"] },
    Emoji { char: "\u{1f61f}", names: &["worried", "concerned"] },
    Emoji { char: "\u{1f641}", names: &["frown", "sad"] },
    Emoji { char: "\u{1f62e}", names: &["open_mouth", "surprised", "wow"] },
    Emoji { char: "\u{1f62f}", names: &["hushed", "surprised"] },
    Emoji { char: "\u{1f632}", names: &["astonished", "shocked", "wow"] },
    Emoji { char: "\u{1f633}", names: &["flushed", "embarrassed", "shocked"] },
    Emoji { char: "\u{1f97a}", names: &["pleading", "puppy_eyes", "please"] },
    Emoji { char: "\u{1f628}", names: &["fearful", "scared", "afraid"] },
    Emoji { char: "\u{1f630}", names: &["anxious", "nervous", "sweat"] },
    Emoji { char: "\u{1f622}", names: &["cry", "sad", "tear"] },
    Emoji { char: "\u{1f62d}", names: &["sob", "crying", "sad", "tears"] },
    Emoji { char: "\u{1f631}", names: &["scream", "scared", "horror"] },
    Emoji { char: "\u{1f624}", names: &["triumph", "proud", "huffing"] },
    Emoji { char: "\u{1f621}", names: &["rage", "angry", "mad"] },
    Emoji { char: "\u{1f620}", names: &["angry", "mad", "grumpy"] },
    Emoji { char: "\u{1f92c}", names: &["cursing", "swearing", "angry"] },
    Emoji { char: "\u{1f608}", names: &["smiling_imp", "devil", "evil"] },
    Emoji { char: "\u{1f47f}", names: &["imp", "devil", "angry"] },
    Emoji { char: "\u{1f480}", names: &["skull", "dead", "death"] },
    Emoji { char: "\u{1f4a9}", names: &["poop", "poo", "shit"] },
    Emoji { char: "\u{1f921}", names: &["clown", "joker"] },
    Emoji { char: "\u{1f47b}", names: &["ghost", "boo", "spooky"] },
    Emoji { char: "\u{1f47d}", names: &["alien", "ufo", "extraterrestrial"] },
    Emoji { char: "\u{1f916}", names: &["robot", "bot", "android"] },
    // Gestures & Body
    Emoji { char: "\u{1f44b}", names: &["wave", "hello", "bye", "hi"] },
    Emoji { char: "\u{1f44c}", names: &["ok", "okay", "perfect"] },
    Emoji { char: "\u{1f90c}", names: &["pinched_fingers", "italian", "chef"] },
    Emoji { char: "\u{270c}\u{fe0f}", names: &["peace", "victory", "v"] },
    Emoji { char: "\u{1f91e}", names: &["crossed_fingers", "luck", "hope"] },
    Emoji { char: "\u{1f918}", names: &["rock", "metal", "horns"] },
    Emoji { char: "\u{1f44d}", names: &["thumbsup", "yes", "good", "like", "+1"] },
    Emoji { char: "\u{1f44e}", names: &["thumbsdown", "no", "bad", "dislike", "-1"] },
    Emoji { char: "\u{1f44f}", names: &["clap", "applause", "bravo"] },
    Emoji { char: "\u{1f64c}", names: &["raised_hands", "hooray", "yay"] },
    Emoji { char: "\u{1f91d}", names: &["handshake", "deal", "agreement"] },
    Emoji { char: "\u{1f64f}", names: &["pray", "please", "thanks", "namaste"] },
    Emoji { char: "\u{1f4aa}", names: &["muscle", "strong", "flex", "bicep"] },
    // Hearts & Love
    Emoji { char: "\u{2764}\u{fe0f}", names: &["heart", "love", "red_heart"] },
    Emoji { char: "\u{1f9e1}", names: &["orange_heart", "heart"] },
    Emoji { char: "\u{1f49b}", names: &["yellow_heart", "heart"] },
    Emoji { char: "\u{1f49a}", names: &["green_heart", "heart"] },
    Emoji { char: "\u{1f499}", names: &["blue_heart", "heart"] },
    Emoji { char: "\u{1f49c}", names: &["purple_heart", "heart"] },
    Emoji { char: "\u{1f5a4}", names: &["black_heart", "heart"] },
    Emoji { char: "\u{1f494}", names: &["broken_heart", "heartbreak", "sad"] },
    // Objects & Symbols
    Emoji { char: "\u{1f525}", names: &["fire", "hot", "lit", "flame"] },
    Emoji { char: "\u{2728}", names: &["sparkles", "stars", "magic", "new"] },
    Emoji { char: "\u{2b50}", names: &["star", "favorite"] },
    Emoji { char: "\u{1f4a5}", names: &["boom", "explosion", "collision"] },
    Emoji { char: "\u{1f4ac}", names: &["speech_bubble", "chat", "comment"] },
    Emoji { char: "\u{1f4ad}", names: &["thought_bubble", "thinking"] },
    Emoji { char: "\u{1f4a4}", names: &["zzz", "sleep", "tired"] },
    Emoji { char: "\u{1f440}", names: &["eyes", "look", "see", "watching"] },
    // Tech & Work
    Emoji { char: "\u{1f4bb}", names: &["laptop", "computer", "mac"] },
    Emoji { char: "\u{1f4f1}", names: &["phone", "iphone", "mobile", "smartphone"] },
    Emoji { char: "\u{1f4e7}", names: &["email", "mail", "envelope"] },
    Emoji { char: "\u{1f4dd}", names: &["memo", "note", "write"] },
    Emoji { char: "\u{1f517}", names: &["link", "chain", "url"] },
    Emoji { char: "\u{1f512}", names: &["lock", "locked", "secure"] },
    Emoji { char: "\u{1f511}", names: &["key", "password"] },
    Emoji { char: "\u{1f527}", names: &["wrench", "tool", "fix"] },
    Emoji { char: "\u{2699}\u{fe0f}", names: &["gear", "settings", "cog"] },
    Emoji { char: "\u{1f4e6}", names: &["package", "box", "shipping"] },
    Emoji { char: "\u{1f4c1}", names: &["folder", "directory"] },
    Emoji { char: "\u{1f4c4}", names: &["document", "file", "page"] },
    Emoji { char: "\u{2705}", names: &["check", "done", "yes", "complete"] },
    Emoji { char: "\u{274c}", names: &["x", "no", "wrong", "cross", "cancel"] },
    Emoji { char: "\u{2753}", names: &["question", "what", "help"] },
    Emoji { char: "\u{2757}", names: &["exclamation", "important", "alert"] },
    Emoji { char: "\u{26a0}\u{fe0f}", names: &["warning", "caution", "alert"] },
    Emoji { char: "\u{1f680}", names: &["rocket", "launch", "ship", "fast"] },
    Emoji { char: "\u{1f389}", names: &["party", "tada", "celebration", "congrats"] },
    Emoji { char: "\u{1f381}", names: &["gift", "present", "birthday"] },
    Emoji { char: "\u{1f3c6}", names: &["trophy", "winner", "award", "champion"] },
    // Weather & Nature
    Emoji { char: "\u{2600}\u{fe0f}", names: &["sun", "sunny", "weather"] },
    Emoji { char: "\u{2601}\u{fe0f}", names: &["cloud", "cloudy", "weather"] },
    Emoji { char: "\u{2744}\u{fe0f}", names: &["snow", "snowflake", "cold", "winter"] },
    Emoji { char: "\u{1f308}", names: &["rainbow", "pride"] },
    Emoji { char: "\u{1f30a}", names: &["wave", "ocean", "water", "sea"] },
    // Food & Drink
    Emoji { char: "\u{2615}", names: &["coffee", "cafe", "hot"] },
    Emoji { char: "\u{1f37a}", names: &["beer", "drink", "alcohol"] },
    Emoji { char: "\u{1f355}", names: &["pizza", "food"] },
    Emoji { char: "\u{1f354}", names: &["burger", "hamburger", "food"] },
    Emoji { char: "\u{1f32e}", names: &["taco", "food", "mexican"] },
    Emoji { char: "\u{1f363}", names: &["sushi", "food", "japanese"] },
    Emoji { char: "\u{1f370}", names: &["cake", "dessert", "birthday"] },
    // Animals
    Emoji { char: "\u{1f436}", names: &["dog", "puppy", "pet"] },
    Emoji { char: "\u{1f431}", names: &["cat", "kitten", "pet"] },
    Emoji { char: "\u{1f430}", names: &["rabbit", "bunny"] },
    Emoji { char: "\u{1f98a}", names: &["fox", "animal"] },
    Emoji { char: "\u{1f43b}", names: &["bear", "animal"] },
    Emoji { char: "\u{1f43c}", names: &["panda", "bear", "animal"] },
    Emoji { char: "\u{1f981}", names: &["lion", "animal", "king"] },
    Emoji { char: "\u{1f427}", names: &["penguin", "animal"] },
    Emoji { char: "\u{1f40d}", names: &["snake", "python", "animal"] },
    Emoji { char: "\u{1f996}", names: &["dinosaur", "trex", "dino"] },
    Emoji { char: "\u{1f419}", names: &["octopus", "sea", "animal"] },
    Emoji { char: "\u{1f42c}", names: &["dolphin", "sea", "animal"] },
    Emoji { char: "\u{1f988}", names: &["shark", "sea", "jaws"] },
    // Arrows & Symbols
    Emoji { char: "\u{2b06}\u{fe0f}", names: &["arrow_up", "up"] },
    Emoji { char: "\u{2b07}\u{fe0f}", names: &["arrow_down", "down"] },
    Emoji { char: "\u{2b05}\u{fe0f}", names: &["arrow_left", "left"] },
    Emoji { char: "\u{27a1}\u{fe0f}", names: &["arrow_right", "right"] },
    Emoji { char: "\u{1f504}", names: &["refresh", "reload", "sync", "arrows"] },
    Emoji { char: "\u{267e}\u{fe0f}", names: &["infinity", "forever"] },
    Emoji { char: "\u{1f4af}", names: &["100", "hundred", "perfect", "score"] },
    Emoji { char: "\u{1f6ab}", names: &["no_entry", "prohibited", "forbidden"] },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_smile() {
        let results = search("smile", 5);
        assert!(!results.is_empty());
        assert!(results
            .iter()
            .any(|e| e.names.iter().any(|n| n.contains("smile"))));
    }

    #[test]
    fn test_search_heart() {
        let results = search("heart", 10);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_empty_query() {
        let results = search("", 5);
        assert_eq!(results.len(), 5);
    }
}
