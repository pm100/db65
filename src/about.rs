use std::collections::HashMap;

use crate::log::say;

pub struct About {
    db: HashMap<String, String>,
    aliases: HashMap<String, String>,
}
impl About {
    pub fn new() -> Self {
        let text = include_str!("about.txt");
        let mut s = Self {
            db: HashMap::new(),
            aliases: HashMap::new(),
        };
        let mut dummy = String::new();
        let mut top_text: &mut String = &mut dummy;
        for line in text.lines() {
            if let Some(topic) = line.strip_prefix('=') {
                let names = topic
                    .split(',')
                    .map(|s| s.to_string())
                    .collect::<Vec<String>>();
                let mut name_iter = names.iter();
                let topic = name_iter.next().unwrap().clone();
                s.db.insert(topic.clone(), String::new());
                for name in name_iter {
                    //  say(&format!("{} is an alias for {}", name, topic));
                    s.aliases.insert(name.to_string(), topic.to_string());
                }
                top_text = s.db.get_mut(&topic).unwrap();
            } else {
                top_text.push_str(line);
                top_text.push('\n');
            }
        }
        s
    }
    pub fn get_topic(&self, topic: &str) -> &str {
        let topic = if let Some(t) = self.aliases.get(topic) {
            t
        } else {
            topic
        };
        self.db
            .get(topic)
            .map(|s| s.as_str())
            .unwrap_or("Unknown topic")
    }
}
