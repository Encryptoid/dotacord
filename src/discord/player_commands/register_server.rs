use poise::CreateReply;
use tracing::info;

use super::super::discord_helper;
use crate::database::{database_access, servers_db};
use crate::{Context, Error};

#[poise::command(slash_command, prefix_command)]
pub async fn register_server(ctx: Context<'_>) -> Result<(), Error> {
    let server_name = discord_helper::guild_name(&ctx)?;

    // Manually get guild_id/send message as we don't want to validate command for this command only
    let guild_id = discord_helper::guild_id(&ctx)?;
    let message = match servers_db::query_server_by_id(guild_id).await? {
        Some(_) => "Server is already registered as a Dotacord server.",
        None => {
            info!(server_name, guild_id, "Registering new discord server");

            let txn = database_access::get_transaction().await?;
            servers_db::insert_server(&txn, guild_id, server_name, None).await?;
            txn.commit().await?;

            "Server has been registered as a Dotacord server."
        }
    }
    .to_string();

    ctx.send(CreateReply::new().content(&message).ephemeral(true))
        .await?;

    let tester = Test {
        my_str: message.clone(),
    };

    test_func(&tester);

    println!("{}", tester.my_str);
    Ok(())
}
struct Test {
    my_str: String,
}

fn test_func(x: &Test) {
    println!("{}", x.my_str);
}
