#![allow(dead_code)]
use std::collections::VecDeque;

use tokio::{task::JoinHandle, io::AsyncWriteExt};
use scraper::{Html, Selector};

const ALPHA_PAGES: &'static[&str] = &[
    "0.html", "A.html", "B.html", "C.html", "D.html", "E.html",
    "F.html", "G.html", "H.html", "I.html", "J.html", "K.html",
    "L.html", "M.html", "N.html", "O.html", "P.html", "Q.html", 
    "R.html", "S.html", "T.html", "U.html", "V.html", "W.html", 
    "X.html", "Y.html", "Z.html"
];

const BASEURL: &str = "https://www.spriters-resource.com";
const ASSETS_DIR_NAME: &str = "assets";

async fn async_get_text(url: String) -> Result<String, reqwest::Error> {
    let content = reqwest::get(url)
        .await?
        .text()
        .await?;
    return Ok(content);
}

fn blocking_get_text(url: String) -> Result<String, reqwest::Error> {
    let content = reqwest::blocking::get(url);
    if content.is_err() {
        return Err(content.err().unwrap());
    }
    return content.unwrap().text();
}

async fn download_asset(url: &String, console_name: &String, game_name: &String, category_name: &String, sprite_name: &String) {
    match reqwest::get(url).await {
        Ok(mut response) => {
            let dirname = format!("{}/{}/{}/{}", ASSETS_DIR_NAME, console_name, game_name, category_name);
            tokio::fs::create_dir_all(&dirname).await.unwrap();
            let extension = response.headers().get(reqwest::header::CONTENT_TYPE).unwrap().to_str().unwrap().split('/').last().unwrap();
            let filename = format!("{}/{}.{}", dirname, sprite_name.replace("/", ""), extension);
            let mut file = tokio::fs::File::create(&filename).await.expect(&filename);
            while let Ok(Some(chunk)) = response.chunk().await {
                // file.write_all(&chunk).await?;
                file.write_all(&chunk).await.unwrap();
            }
        },
        Err(error) => {
            println!("ERR: {}", error);
            return;
        }
    }
}

async fn scrape_sprite_sheet(baseurl: String, sheet_href: String, console_name: String, game_name: String, category: String, sheet_name: String) -> Option<JoinHandle<()>>{
    // get the sprite's png reference
    let html_sprite_page = match async_get_text(baseurl.clone() + &sheet_href).await {
        Ok(content) => content,
        Err(error) => {
            println!("{}", error);
            return None;
        }
    };

    let sprite_document = Html::parse_document(&html_sprite_page);
    let img_selector = Selector::parse("#sheet-container > a > img").unwrap();
    
    let sprite_src;
    let sprite_img_option = sprite_document.select(&img_selector).next();
    if sprite_img_option.is_none() {
        let zip_selector = Selector::parse("#content > a").unwrap(); // get the first link inside content
        let sprite_zip_option = sprite_document.select(&zip_selector).next();
        if sprite_zip_option.is_none() {
            println!("ERR: {} has no known img or download link", sheet_href);
            return None;
        }
        let sprite_zip_href_option = sprite_zip_option.unwrap().value().attr("href");
        if sprite_zip_href_option.is_none() {
            println!("ERR: {} zip download anchor has no href attr", sheet_href);
            return None;
        }
        sprite_src = sprite_zip_href_option.unwrap();
    } else {
        let sprite_img_src_option = sprite_img_option.unwrap().value().attr("src");
        if sprite_img_src_option.is_none() {
            println!("ERR: {} img has no attr src", sheet_href);
            return None;
        }

        sprite_src = sprite_img_src_option.unwrap();
    }

    let sprite_src_str = format!("{}{}", baseurl, sprite_src);
    let handle = tokio::spawn(async move {
        download_asset(&sprite_src_str, &console_name, &game_name, &category, &sheet_name).await;
        println!("{}/{}/{}/{}", &console_name, &game_name, &category, &sheet_name);
    });
    
    return Some(handle);
}

async fn scrape_game_page(baseurl: &String, game_href: &String, console_name: &String, game_name: &String) -> Vec<JoinHandle<()>> {
    let mut join_handles = Vec::new();
    let html_game_page = match async_get_text(String::from(baseurl) + &game_href).await {
        Ok(content) => content,
        Err(error) => {
            println!("{}", error);
            return join_handles;
        }
    };

    let game_document = Html::parse_document(&html_game_page);

    let mut categories: VecDeque<String> = VecDeque::new();
    let cat_title_selector = Selector::parse("div.sect-name").unwrap();

    for cat_title in game_document.select(&cat_title_selector) {
        let title_str = cat_title.to_owned().value().attr("title").unwrap();
        categories.push_back(title_str.to_string());
    }

    let sprite_group_selector = Selector::parse("div.updatesheeticons").unwrap();
    let sprite_link_selector = Selector::parse("a").unwrap();
    for sprite_group in game_document.select(&sprite_group_selector) {
        let category = categories.pop_front();
        let category_name: String;
        if category.is_none() {
            category_name = "".to_string();
        } else {
            category_name = category.unwrap();
        }

            
        for sheet_container in sprite_group.select(&sprite_link_selector) {
            let sheet_href_option = sheet_container.value().attr("href");
            if sheet_href_option.is_none(){
                continue;
            }

            let sheet_href = sheet_href_option.unwrap();
            let sheet_name_selector = Selector::parse("span.iconheadertext").unwrap();

            let mut sheet_name_option = sheet_container.select(&sheet_name_selector);
            let sheet_name = sheet_name_option.next().unwrap().inner_html();
            
            let sheet_href_str = String::from(sheet_href);
            let base_url_str = String::from(BASEURL);
            let console_str = console_name.clone();
            let game_str = game_name.clone();
            let cat_str = category_name.clone();
            let handle = tokio::spawn(async move {
                let result = scrape_sprite_sheet(base_url_str, sheet_href_str, console_str, game_str, cat_str, sheet_name.clone()).await;
                if result.is_none() {
                }

                let join_result = result.unwrap().await;
                if join_result.is_err() {
                    println!("ERR: scrape_sprite_sheet did not join properly");
                    return;
                } else {
                    join_result.unwrap();
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            });
            join_handles.push(handle);

        }
    } 

    return join_handles;
}

async fn archive_console(console_name: &str) {
    for alpha_page in ALPHA_PAGES {
        let page_url = format!("{}/{}/{}", BASEURL, console_name, alpha_page);
        let html_console_page = match async_get_text(page_url).await {
            Ok(content) => content,
            Err(error) => {
                println!("{}", error);
                return;
            }
        };

        let console_document = Html::parse_document(&html_console_page);

        let game_link_selector = Selector::parse("#content > \
                                                div:nth-child(4) a").unwrap();
        let game_name_selector = Selector::parse("span.gameiconheadertext").unwrap();

        let mut join_handles = Vec::new();

        for element in console_document.select(&game_link_selector){
            let game_href_option = element.value().attr("href");
            if game_href_option.is_none(){
                continue;
            }
            let game_href = game_href_option.unwrap();

            let game_name_option = element.select(&game_name_selector).next();
            let game_name;
            if game_name_option.is_none() {
                game_name = game_href.trim_end_matches('/').split('/').last().unwrap().to_string();
            } else {
                game_name = game_name_option.unwrap().inner_html().replace("/", "");
            }

            let game_href_str = String::from(game_href);
            let base_url_str = String::from(BASEURL);
            let console_str = String::from(console_name);
            let handle = tokio::spawn( async move {
                let sub_handles = scrape_game_page(&base_url_str, &game_href_str, &console_str, &game_name).await;
                for handle in sub_handles {
                    let result = handle.await;
                    if result.is_err() {
                        println!("error when joining sprite scrape {}", result.err().unwrap())
                    } else {
                        result.unwrap();
                    }
                }
            });

            // tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
            join_handles.push(handle);

        }

        for handle in join_handles {
            handle.await.unwrap();
        }
    }

}

async fn archive_single_game() {

    let game_href_str = String::from("/game_boy_advance/cvcom/");
    let base_url_str = String::from(BASEURL);
    let handle = tokio::spawn( async move {
        let sub_handles = scrape_game_page(&base_url_str, &game_href_str, &"game_boy_advance".to_string(), &"cvcom".to_string()).await;
        for sub in sub_handles {
            let result = sub.await;
            if result.is_err() {
                println!("error when joining sprite scrape {}", result.err().unwrap())
            } else {
                result.unwrap();
            }
        }
    });

    handle.await.unwrap();
}

async fn archive_single_sprite() {
    let sprite_href_str = String::from("/pc_computer/diablodiablohellfire/sheet/65453/");
    let base_url_str = String::from(BASEURL);
    scrape_sprite_sheet(base_url_str, sprite_href_str, "pc_computer".to_string(), "diablodiablohellfire".to_string(), "Characters".to_string(), "Warrior".to_string()).await;
}

#[tokio::main]
async fn main() {
    tokio::join!(
        archive_console("nes")
        // archive_single_console_letter()
        // archive_single_game()
        // archive_single_sprite()
        // download_asset("https://www.spriters-resource.com/resources/sheets/150/153616.png?updated=1618030124".to_string(),
        //                "pc_computer".to_string(), "stardewvalley".to_string(), "Fishing".to_string())
    );
}
