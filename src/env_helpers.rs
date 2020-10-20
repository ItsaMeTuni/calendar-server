use std::env;

pub fn get_env(name: &str) -> String
{
    env::vars()
        .find(|(key, _)| key == name)
        .expect(&format!("Missing {} environment variable.", name))
        .1
}

pub fn get_env_default(name: &str, default: &str) -> String
{
    env::vars()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value)
        .unwrap_or(default.to_owned())
}