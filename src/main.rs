
use scraper::{Html, Selector};

async fn async_get_content(url: String) -> Result<String, reqwest::Error> {
    let content = reqwest::get(url)
        .await?
        .text()
        .await?;
    return Ok(content);
}

async fn scrape_game_page(baseurl: String, game_href: String) {
    let html_game_page = match async_get_content(baseurl + &game_href).await {
        Ok(content) => content,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    let game_document = Html::parse_document(&html_game_page);
    let sheet_selector = Selector::parse("div.updatesheeticons > a").unwrap();

    println!("\n{}", game_href);
    for sheet_container in game_document.select(&sheet_selector) {
        let sheet_href_option = sheet_container.value().attr("href");
        if sheet_href_option.is_none(){
            continue;
        }
        let sheet_href = sheet_href_option.unwrap();
        let sheet_name_selector = Selector::parse("span.iconheadertext").unwrap();

        let mut sheet_name_option = sheet_container.select(&sheet_name_selector);
        let sheet_name = sheet_name_option.next().unwrap().inner_html();

        println!("\t{:60}{}", sheet_name, sheet_href);

    }
}

async fn run() {
    let baseurl = String::from("https://www.spriters-resource.com");

    let html_console_page = match async_get_content(baseurl.clone() + "/pc_computer/C.html").await {
        Ok(content) => content,
        Err(error) => {
            println!("{}", error);
            return;
        }
    };

    let console_document = Html::parse_document(&html_console_page);

    let game_link_selector = Selector::parse("#content > \
                                              div:nth-child(4) a").unwrap();
                                    // div.gameiconcontainer > \
                                    // div.gameiconheader > \
                                    // span.gameiconheadertext").unwrap();

    let mut join_handles = Vec::new();

    for element in console_document.select(&game_link_selector){
        let game_href_option = element.value().attr("href");
        if game_href_option.is_none(){
            continue;
        }

        let game_href = game_href_option.unwrap();
        let game_href_str = String::from(game_href);
        let base_url_str = baseurl.clone();
        let handle = tokio::spawn( async move {
            scrape_game_page(base_url_str, game_href_str.clone()).await;
        });
        join_handles.push(handle);

    }

    for handle in join_handles {
        handle.await.unwrap();
    }

}

#[tokio::main]
async fn main() {
    tokio::join!(
        run()
    );
}
