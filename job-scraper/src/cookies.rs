pub(crate) fn encode_cookies<I>(pairs: I) -> String
where
    I: Iterator<Item = (String, String)>,
{
    let encoded_cookies = pairs
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join(";");
    return encoded_cookies;
}
