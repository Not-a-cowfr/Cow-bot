use crate::commands::checkPlayer::{get_linked_elite_account, get_mojang_info};
use crate::Error;

pub async fn get_account_from_anything(identifier: String) -> Result<(String, String), Error> {
    let (uuid, username);
    if (identifier.len() == 32) | (identifier.len() <= 16) {
        // mojang uuid or username
        let result = get_mojang_info(identifier.into()).await?;
        username = result.0;
        uuid = result.1;
    } else if identifier
        .replace(&['@', '<', '>'][..], "")
        .trim()
        .parse::<u64>()
        .is_ok()
    {
        // discord id
        let result = get_linked_elite_account(identifier.into()).await?;
        username = result.0;
        uuid = result.1;
    } else {
        return Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid player name or UUID",
        )));
    }
    Ok((username, uuid))
}
