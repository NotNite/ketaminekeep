use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum Face {
    South = 0,
    East = 1,
    North = 2,
    West = 3,
    Bottom = 4,
    Top = 5,
}

pub fn parse_properties(props: &str) -> HashMap<String, String> {
    // remove []
    let props = props.trim_matches(|c| c == '[' || c == ']');
    let mut properties = HashMap::new();

    if props.trim() == "" {
        return properties;
    }

    for prop in props.split(',') {
        let mut prop = prop.split('=');
        let key = prop.next().unwrap().to_string();
        let value = prop.next().unwrap().to_string();
        properties.insert(key, value);
    }

    properties
}
