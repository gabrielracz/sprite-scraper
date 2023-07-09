#![allow(dead_code)]
use tokio::task::JoinHandle;
use scraper::{Html, Selector};

const BASEURL: &str = "https://www.spriters-resource.com";

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

async fn scrape_sprite_sheet(baseurl: String, sheet_href: String, sheet_name: String){
    // get the sprite's png reference
    let html_sprite_page = match async_get_text(baseurl.clone() + &sheet_href).await {
        Ok(content) => content,
        Err(error) => {
            println!("{}", error);
            return ;
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
            return;
        }
        let sprite_zip_href_option = sprite_zip_option.unwrap().value().attr("href");
        if sprite_zip_href_option.is_none() {
            println!("ERR: {} zip download anchor has no href attr", sheet_href);
            return;
        }
        sprite_src = sprite_zip_href_option.unwrap();
    } else {
        let sprite_img_src_option = sprite_img_option.unwrap().value().attr("src");
        if sprite_img_src_option.is_none() {
            println!("ERR: {} img has no attr src", sheet_href);
            return ;
        }

        sprite_src = sprite_img_src_option.unwrap();
    }
    
    println!("\t{:60}{}", sheet_name, sprite_src);
}

async fn scrape_game_page(baseurl: String, game_href: String) -> Vec<JoinHandle<()>> {
    let mut join_handles = Vec::new();
    let html_game_page = match async_get_text(String::from(&baseurl) + &game_href).await {
        Ok(content) => content,
        Err(error) => {
            println!("{}", error);
            return join_handles;
        }
    };

    let game_document = Html::parse_document(&html_game_page);
    let sheet_selector = Selector::parse("div.updatesheeticons > a").unwrap();

    // println!("\n{}", game_href);
    for sheet_container in game_document.select(&sheet_selector) {
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
        let handle = tokio::spawn(async move {
            scrape_sprite_sheet(base_url_str, sheet_href_str, sheet_name.clone()).await;
        });
        join_handles.push(handle);
    }
    return join_handles;
}

async fn archive_single_console_letter() {
    let html_console_page = match async_get_text(String::from(BASEURL) + "/pc_computer/C.html").await {
        Ok(content) => content,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    let console_document = Html::parse_document(&html_console_page);

    let game_link_selector = Selector::parse("#content > \
                                              div:nth-child(4) a").unwrap();

    let mut join_handles = Vec::new();

    for element in console_document.select(&game_link_selector){
        let game_href_option = element.value().attr("href");
        if game_href_option.is_none(){
            continue;
        }
        let game_href = game_href_option.unwrap();
        let game_href_str = String::from(game_href);
        let base_url_str = String::from(BASEURL);
        let handle = tokio::spawn( async move {
            let sub_handles = scrape_game_page(base_url_str, game_href_str.clone()).await;
            for handle in sub_handles {
                let result = handle.await;
                if result.is_err() {
                    println!("error when joining sprite scrape {}", result.err().unwrap())
                };
            }
        });

        // tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        join_handles.push(handle);

    }

    for handle in join_handles {
        handle.await.unwrap();
    }

}

async fn archive_single_game() {

    let game_href_str = String::from("/pc_computer/diablodiablohellfire/");
    let base_url_str = String::from(BASEURL);
    let handle = tokio::spawn( async move {
        let sub_handles = scrape_game_page(base_url_str, game_href_str.clone()).await;
        for handle in sub_handles {
            let result = handle.await;
            if result.is_err() {
                println!("error when joining sprite scrape {}", result.err().unwrap())
            };
        }
    });

    handle.await.unwrap();
}

async fn archive_single_sprite() {
    let sprite_href_str = String::from("/pc_computer/diablodiablohellfire/sheet/65453/");
    let base_url_str = String::from(BASEURL);
    scrape_sprite_sheet(base_url_str, sprite_href_str, "Warrior".to_string()).await;
}

#[tokio::main]
async fn main() {
    tokio::join!(
        archive_single_console_letter()
        // archive_single_game()
        // archive_single_sprite()
    );
}
