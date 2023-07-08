use std::sync::{Arc, Mutex};
use std::thread;
use std::fs::File;
// use std::io::Write;
use reqwest::StatusCode;

fn test_url(url: &str, sheet: i32, page: i32) -> bool {
    match reqwest::blocking::get(url) {
        Ok(mut response) => {
            let urlok: bool = response.status() == StatusCode::OK;
            if urlok {
                println!("BOOM {0} {1} {2} {3:?}", sheet, page, 200, response.content_length());
                let mut file = File::create(format!("valid-paths/{sheet}-{page}")).unwrap();
                response.copy_to(&mut file).unwrap();
            } else {
                println!("FAIL {} {} {}", sheet, page, response.status())
            }
            return urlok;
        },
        Err(err) => {
            println!("ERR {0} {1} {2}", sheet, page, err);
            return false;
        }
    }
}

fn main() {
    let mut urls: Vec<(String, i32, i32)> = vec![];

    const BASEURL: &str = "https://www.spriters-resource.com/resources/sheets";

    for i in (0..100).rev() {
        for j in (0..100000).rev().step_by(100) {
            let resource_location: String = format!("{}/{}/{}.gif", BASEURL, i, j);
            let triple: (String, i32, i32) = (resource_location, i, j);
            urls.push(triple);
        }
    }

    let urls_arc = Arc::new(Mutex::new(urls));

    let mut handles = vec![];

    for _ in 0..8 { // Number of parallel threads
        let urls = Arc::clone(&urls_arc);
        let handle = thread::spawn(move || {
            loop {
                let triple: (String, i32, i32);
                {
                    let mut urls = urls.lock().unwrap();
                    if urls.is_empty() {

                        break;
                    }
                    triple = urls.pop().unwrap();
                }
                let (url, sheet, page) = triple;
                test_url(&url, sheet, page);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
