use postgres::{ Client, NoTls };
use postgres::Error as PostgresError;
use std::net::{ TcpListener, TcpStream };
use std::io::{ Read, Write };
use std::env;
use http::header::ACCESS_CONTROL_ALLOW_ORIGIN;

#[macro_use]
extern crate serde_derive;

//Model: User struct with id, name, email
#[derive(Serialize, Deserialize)]
struct Laptop {
    id: Option<i32>, // Change to non-optional i32
    name: String,
    description: String,    
    price: String,
    processor: String,
    ram: String,
    storage: String,
    display: String,
    os: String,
    graphics: String,
}

//DATABASE URL
const DB_URL: &str = env!("DATABASE_URL");

//constants
const OK_RESPONSE: &str = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n";
const NOT_FOUND: &str = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
const INTERNAL_ERROR: &str = "HTTP/1.1 500 INTERNAL ERROR\r\n\r\n";

//main function
fn main() {

    //Set Database
    if let Err(_) = set_database() {
        println!("Error setting database");
        return;
    }

    //start server and print port
    let listener = TcpListener::bind(format!("0.0.0.0:6001")).unwrap();
    println!("Server listening on port 6001");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream);
            }
            Err(e) => {
                println!("Unable to connect: {}", e);
            }
        }
    }
}

//handle requests
fn handle_client(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    let mut request = String::new();

    match stream.read(&mut buffer) {
        Ok(size) => {
            request.push_str(String::from_utf8_lossy(&buffer[..size]).as_ref());

            let (status_line, content) = match &*request {
                r if r.starts_with("POST /laptops") => handle_post_request(r),
                r if r.starts_with("GET /laptops/") => handle_get_request(r),
                r if r.starts_with("GET /laptops") => handle_get_all_request(r),
                r if r.starts_with("PUT /laptops/") => handle_put_request(r),
                r if r.starts_with("DELETE /laptops/") => handle_delete_request(r),
                _ => (NOT_FOUND.to_string(), "404 not found".to_string()),
            };

            stream
                .write_all(format!("{}{}", status_line, content).as_bytes())
                .unwrap();
            stream
                .write_all(
                    format!(
                        "{}: {}\r\n",
                        ACCESS_CONTROL_ALLOW_ORIGIN,
                        "http://localhost:5173" // Replace with your allowed origins
                    )
                    .as_bytes(),
                )
                .unwrap();
        }
        Err(e) => eprintln!("Unable to read stream: {}", e),
    }
}

//handle post request
fn handle_post_request(request: &str) -> (String, String) {
    match (get_user_request_body(&request), Client::connect(DB_URL, NoTls)) {
        (Ok(laptop), Ok(mut client)) => {
            match client.execute(
                "INSERT INTO laptops (name, description, price, processor, ram, storage, display, os, graphics) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
                &[
                    &laptop.name,
                    &laptop.description,
                    &laptop.price,
                    &laptop.processor,
                    &laptop.ram,
                    &laptop.storage,
                    &laptop.display,
                    &laptop.os,
                    &laptop.graphics,
                ],
            ) {
                Ok(_) => (OK_RESPONSE.to_string(), "Laptop created".to_string()),
                Err(e) => {
                    eprintln!("Error executing SQL query: {:?}", e);
                    (INTERNAL_ERROR.to_string(), "Internal error".to_string())
                }
            }
        }
        (Err(_), _) | (_, Err(_)) => {
            eprintln!("Error connecting to the database");
            (INTERNAL_ERROR.to_string(), "Internal error".to_string())
        }
    }
}


//handle get request
fn handle_get_request(request: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), Client::connect(DB_URL, NoTls)) {
        (Ok(id), Ok(mut client)) =>
            match client.query_one("SELECT * FROM laptops WHERE id = $1", &[&id]) {
                Ok(row) => {
                    let laptop = Laptop { 
                        id: row.get(0), 
                        name: row.get(1), 
                        description: row.get(2), 
                        price: row.get(3), 
                        processor: row.get(4), 
                        ram: row.get(5), 
                        storage: row.get(6), 
                        display: row.get(7), 
                        os: row.get(8), 
                        graphics: row.get(9)     
                    };

                    (OK_RESPONSE.to_string(), serde_json::to_string(&laptop).unwrap())
                }
                _ => (NOT_FOUND.to_string(), "Laptop not found".to_string()),
            }

        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

//handle get all request
fn handle_get_all_request(_request: &str) -> (String, String) {
    match Client::connect(DB_URL, NoTls) {
        Ok(mut client) => {
            let mut 
            laptops = Vec::new();

            for row in client.query("SELECT id, name, description, price, processor, ram, storage, display, os, graphics FROM laptops", &[]).unwrap() {
                laptops.push(Laptop {
                    id: row.get(0), 
                    name: row.get(1), 
                    description: row.get(2), 
                    price: row.get(3), 
                    processor: row.get(4), 
                    ram: row.get(5), 
                    storage: row.get(6), 
                    display: row.get(7), 
                    os: row.get(8), 
                    graphics: row.get(9)   
                });
            }

            (OK_RESPONSE.to_string(), serde_json::to_string(&laptops).unwrap())
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

//handle put request
fn handle_put_request(request: &str) -> (String, String) {
    match
        (
            get_id(&request).parse::<i32>(),
            get_user_request_body(&request),
            Client::connect(DB_URL, NoTls),
        )
    {
        (Ok(id), Ok(laptop), Ok(mut client)) => {
            client
                .execute(
                    "UPDATE laptops SET name = $1, description = $2, price = $3, processor = $4, ram = $5, storage = $6, display = $7, os = $8, graphics = $9 WHERE id = $10",
                    &[
                        &laptop.name,
                        &laptop.description,
                        &laptop.price,
                        &laptop.processor,
                        &laptop.ram,
                        &laptop.storage,
                        &laptop.display,
                        &laptop.os,
                        &laptop.graphics,
                        &laptop.id,
                    ],
                )
                .unwrap();

            (OK_RESPONSE.to_string(), "Laptop updated".to_string())
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

//handle delete request
fn handle_delete_request(request: &str) -> (String, String) {
    match (get_id(&request).parse::<i32>(), Client::connect(DB_URL, NoTls)) {
        (Ok(id), Ok(mut client)) => {
            let rows_affected = client.execute("DELETE FROM laptops WHERE id = $1", &[&id]).unwrap();

            //if rows affected is 0, user not found
            if rows_affected == 0 {
                return (NOT_FOUND.to_string(), "Laptop not found".to_string());
            }

            (OK_RESPONSE.to_string(), "Laptop deleted".to_string())
        }
        _ => (INTERNAL_ERROR.to_string(), "Internal error".to_string()),
    }
}

//db setup
fn set_database() -> Result<(), PostgresError> {
    let mut client = Client::connect(DB_URL, NoTls)?;
    client.batch_execute(
        "CREATE TABLE IF NOT EXISTS laptops (
            id SERIAL PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            price TEXT NOT NULL,
            processor TEXT NOT NULL,
            ram TEXT NOT NULL,
            storage TEXT NOT NULL,
            display TEXT NOT NULL,
            os TEXT NOT NULL,
            graphics TEXT NOT NULL
        )"
    )?;
    Ok(())
}

//Get id from request URL
fn get_id(request: &str) -> &str {
    request.split("/").nth(2).unwrap_or_default().split_whitespace().next().unwrap_or_default()
}

//deserialize user from request body without id
fn get_user_request_body(request: &str) -> Result<Laptop, serde_json::Error> {
    serde_json::from_str(request.split("\r\n\r\n").last().unwrap_or_default())
}