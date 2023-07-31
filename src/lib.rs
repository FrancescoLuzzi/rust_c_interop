use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::os::raw::{c_char, c_uint};
use tiberius::{error::Error, AuthMethod, Client, Config};
use tokio::net::TcpStream;
use tokio::runtime::Runtime;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

// https://stackoverflow.com/questions/68830056/the-proper-method-to-get-tokio-runtime-handle-based-on-current-running-environme
static RUNTIME: Lazy<Runtime> = Lazy::new(|| Runtime::new().unwrap());

// wrap Client in tuple to generate header file correctly
#[allow(non_camel_case_types)]
pub struct client_t(Client<Compat<TcpStream>>);

fn get_client(client: *mut client_t) -> &'static mut Client<Compat<TcpStream>> {
    assert!(!client.is_null());
    // get client inside tuple
    unsafe { &mut (*client).0 }
}

fn init_db_config(host: &str, port: u16, username: &str, password: &str, database: &str) -> Config {
    let mut config = Config::new();
    config.host(host);
    config.port(port);
    config.authentication(AuthMethod::sql_server(username, password));
    config.database(database);
    config.trust_cert();
    config
}

#[no_mangle]
pub unsafe extern "C" fn init_connection(
    host: *const c_char,
    port: c_uint,
    username: *const c_char,
    password: *const c_char,
    database: *const c_char,
) -> *mut client_t {
    let r_host = CStr::from_ptr(host).to_str().unwrap();
    let r_username = CStr::from_ptr(username).to_str().unwrap();
    let r_password = CStr::from_ptr(password).to_str().unwrap();
    let r_database = CStr::from_ptr(database).to_str().unwrap();
    let tokio_rt = RUNTIME.handle();
    let client = tokio_rt.block_on(async {
        let config = init_db_config(r_host, port as u16, r_username, r_password, r_database);
        let tcp = TcpStream::connect(config.get_addr()).await.unwrap();
        tcp.set_nodelay(true).unwrap();

        match Client::connect(config, tcp.compat_write()).await {
            // Connection successful.
            Ok(client) => Ok(client),
            // The server wants us to redirect to a different address
            Err(Error::Routing { host, port }) => {
                let config = init_db_config(&host, port, r_username, r_password, r_database);
                let tcp = TcpStream::connect(config.get_addr()).await.unwrap();
                tcp.set_nodelay(true).unwrap();

                // we should not have more than one redirect, so we'll short-circuit here.
                Client::connect(config, tcp.compat_write()).await
            }
            // Generic error
            Err(e) => Err(e),
        }
    });

    // allocate on heap a tuple with Client inside
    // then deref it giving ownership to the user
    Box::into_raw(Box::new(client_t(client.unwrap())))
}

#[no_mangle]
pub unsafe extern "C" fn simple_query(client: *mut client_t, table: *const c_char) -> bool {
    let r_table = {
        assert!(!table.is_null());
        CStr::from_ptr(table).to_str().unwrap()
    };
    let r_client = get_client(client);
    let tokio_rt = RUNTIME.handle();
    let query_output = tokio_rt.block_on(async {
        let query = r_client
            .simple_query(format!("SELECT * FROM {r_table}"))
            .await;
        if query.is_err() {
            return None;
        }
        let query = query.unwrap().into_row().await;
        match query {
            Ok(o) => o,
            Err(_) => None,
        }
    });
    query_output.is_some()
}

#[no_mangle]
pub unsafe extern "C" fn free_client(client: *mut client_t) {
    if client.is_null() {}
    let _ = Box::from_raw(client);
}
