use tiberius::{error::Error, AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio_util::compat::TokioAsyncWriteCompatExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = Config::new();

    config.host("<Host>");
    config.port(1433);
    config.authentication(AuthMethod::sql_server("SA", "<VerySecurePassword>"));
    config.database("Master");
    config.trust_cert();

    let tcp = TcpStream::connect(config.get_addr()).await?;
    tcp.set_nodelay(true)?;

    let mut client = match Client::connect(config, tcp.compat_write()).await {
        // Connection successful.
        Ok(client) => client,
        // The server wants us to redirect to a different address
        Err(Error::Routing { host, port }) => {
            let mut config = Config::new();

            config.host(&host);
            config.port(port);
            config.authentication(AuthMethod::sql_server("SA", "<VerySecurePassword>"));
            config.database("Master");
            config.trust_cert();

            let tcp = TcpStream::connect(config.get_addr()).await?;
            tcp.set_nodelay(true)?;

            // we should not have more than one redirect, so we'll short-circuit here.
            Client::connect(config, tcp.compat_write()).await?
        }
        Err(e) => Err(e)?,
    };

    let res = client
        .simple_query("SELECT @@version as \"ver\"")
        .await?
        .into_row()
        .await
        .expect("nessun risultato trovato");
    if let Some(t) = res {
        let out: Result<Option<&str>,Error> = t.try_get("ver");
        print!(
            "{}",
            match out {
                Ok(Some(o)) => format!("found {}", o),
                Ok(None) => "colonna vuota".to_string(),
                Err(_) => "colonna non trovata".to_string(),
            }
        );
    }
    Ok(())
}
