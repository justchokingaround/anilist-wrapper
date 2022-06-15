// This example uses 3 crates serde_json, reqwest, tokio

use serde_json::json;
use reqwest::Client;
use std::io::{self, Write};

const QUERY: &str = "
query ($id: Int, $page: Int, $perPage: Int, $search: String) {
    Page (page: $page, perPage: $perPage) {
        pageInfo {
            total
            currentPage
            lastPage
            hasNextPage
            perPage
        }
        media (id: $id, search: $search, type: ANIME) {
            id
            title {
                romaji
            }
        }
        media (id: $id, search: $search, type: ANIME) {
            id
            coverImage {
                extraLarge
            }
        }
        media (id: $id, search: $search, type: ANIME) {
            id
            description
        }
    }
}
";

#[tokio::main]
async fn main() {

    // if the user gives an argument, assign it to the input variable, otherwise ask for input
    let input = if let Some(arg) = std::env::args().nth(1) {
        arg
    } else {
        let mut input = String::new();
        print!("Please enter the anime name: ");
        let _ = io::stdout().flush();
        io::stdin().read_line(&mut input).expect("Error reading from STDIN");
        input.trim().to_string()
    };

    let client = Client::new();

    let json = json!(
        {
            "query": QUERY,
            "variables": {
                "search": input.trim(),
                "page": 1,
                "perPage": 11
            }
        }
    );

    let resp = client.post("https://graphql.anilist.co/")
                .header("Content-Type", "application/json")
                .header("Accept", "application/json")
                .body(json.to_string())
                .send()
                .await
                .unwrap()
                .text()
                .await;

    let result: serde_json::Value = serde_json::from_str(&resp.unwrap()).unwrap();

    let mut urls: Vec<String> = Vec::new();
    for i in 0..result["data"]["Page"]["media"].as_array().unwrap().len() {
        urls.push(result["data"]["Page"]["media"][i]["coverImage"]["extraLarge"].as_str().unwrap().to_string());
    }

    // display the results preceded by the index
    for i in 0..result["data"]["Page"]["media"].as_array().unwrap().len() {
        println!("[{}] {}",i, result["data"]["Page"]["media"][i]["title"]["romaji"].as_str().unwrap());
    }

    // ask the user to select an anime and if the input is empty set it to 0
    let mut index = String::new();
    print!("Please enter the anime number: ");
    let _ = io::stdout().flush();
    io::stdin().read_line(&mut index).expect("Error reading from STDIN");
    let index = index.trim().parse::<usize>().unwrap_or(0);
    println!("");

    // display the anime's cover image if --thumbnail or -t or --cover is passed as an argument
    if let Some(arg) = std::env::args().nth(2) {
        if arg == "--thumbnail" || arg == "-t" || arg == "--cover" {
            // if pixcat is installed, use it to display the image
            if let Ok(_output) = std::process::Command::new("pixcat").arg(urls[index].clone()).output() {
                let mut cmd = std::process::Command::new("pixcat");
                cmd.arg("thumbnail")
                    .arg("--size")
                    .arg("1080")
                    .arg("--align")
                    .arg("left")
                    .arg(urls[index].as_str());
                cmd.spawn().unwrap();
                std::thread::sleep(std::time::Duration::from_secs(1));
            } else {
                // if pixcat is not installed, tell the user to install it and exit
                println!("\npixcat is not installed. Please install it to display the cover image.");
                println!("To install pixcat, run the following command:\npip install pixcat");
                std::process::exit(1);
            }
        }
    }

    println!("{}", result["data"]["Page"]["media"][index]["description"].as_str().unwrap()
        .replace("<br><br>", "\n")
        .replace("<br>", "")
        .replace("<i>", "").replace("</i>", ""));

}