use reqwest;
use colored::*;
use tokio::task;
use indicatif::{ProgressBar,ProgressStyle};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use scraper::{Html, Selector};
use chrono::{DateTime, Local, TimeZone};
use std::io::{Write};
use std::fs::{File};
use blogworm::Postsrc;
use blogworm::{Post};
use super::time::parse_time;
use serde_json;

pub async fn send_request(url: String) -> Result<String, Box<dyn std::error::Error>> {
    let reponse = reqwest::get(url.to_string()).await?;
    let body = reponse.text().await?;
    Ok(body)
}

pub fn parse_postsrc(document_body: &str, link_class: &str, post_id: u32) -> Result<Vec<String>, Box<dyn std::error::Error>>{
    if post_id == 2 {
        let document = Html::parse_document(&document_body);
        let div_selector = Selector::parse("div.link").unwrap();
        let a_selector = Selector::parse("a").unwrap();
        let a_values: Vec<String> = document
            .select(&div_selector)
            .flat_map(|div| div.select(&a_selector))
            .filter_map(|a| a.value().attr("href").map(String::from))
            .collect();

		Ok(a_values)
    }else{
        let document = Html::parse_document(&document_body);
        let a_selector = Selector::parse(&format!("a.{}", link_class)).unwrap();
        let a_values: Vec<String> = document 
            .select(&a_selector)
            .filter_map(|a| a.value().attr("href").map(String::from))
            .collect();
		Ok(a_values)
  
    }
}

pub fn parse_post(document_body: &str, time_class: &str, title_class: &str, author_class: &str, content_class: &str, post_id: u32, post_url: String) -> Result<Post, Box<dyn std::error::Error>>{
    let document = Html::parse_document(&document_body);
    let time_values: Vec<String> = handle_parse_post(time_class, &document);
    let title_values: Vec<String> = handle_parse_post(title_class, &document);
    let author_values: Vec<String> = handle_parse_post(author_class, &document);
    let content_values: Vec<String> = handle_parse_post(content_class, &document);
    let parsed_time = parse_time(post_id, time_values[0].clone());
    Ok(Post {content: content_values[0].clone(), author: author_values[0].clone(), title: title_values[0].clone(), create_timestamp: parsed_time, url: post_url})
}
pub fn handle_parse_post(class_name: &str, document: &Html) -> Vec<String>{
    let abc_selector = Selector::parse(class_name).unwrap();
    document.select(&abc_selector).map(|a| a.inner_html()).collect()
    
}
pub async fn get_blog_link_from_postsrc(postsrc: &Postsrc) -> Result<(String, Vec<String>), Box<dyn std::error::Error>>{
    let website = &postsrc.website;
    let link_class = &postsrc.link_class;
    let body = send_request(website.to_string()).await;

    match body {
          Ok(document_body) => {
            let a_link_list  = parse_postsrc(document_body.as_str(), link_class.as_str(), postsrc.postsrc_id);
            match a_link_list {
                Ok(a_values) => {
                    Ok((website.clone(), a_values))
                }
                Err(error) => {
                    println!("[!] Fail to parse HTML.\n Error: {}", error);
                    return Err(error);
                }
            }
          }
          Err(error) => {
                eprintln!("Error: {}",error);
                return Err(error);
          }

    }

}

pub async fn get_post_from_link(post_url: String, postsrc: &Postsrc) -> Result<Post, Box<dyn std::error::Error>>{
    let body = send_request(post_url.to_string()).await;
    match body {
        Ok(document_body) => {
            let post = parse_post(document_body.as_str(), postsrc.time_class.as_str(), postsrc.title_class.as_str(), postsrc.author_class.as_str(), postsrc.content_class.as_str(), postsrc.postsrc_id, post_url).unwrap();
            Ok(post)

        }
        Err(error) => {
            println!("[!] Failt to parse POST HTML.\n Error: {}",error);
            return Err(error);
        }

    }
}

#[warn(deprecated)]
pub fn timestamp_to_readable(timestamp: u64) -> DateTime<Local>{
    let timestamp = chrono::NaiveDateTime::from_timestamp(timestamp as i64, 0);
    Local.from_utc_datetime(&timestamp)
}


pub fn save_new_post_to_file(new_post_list: Vec<Post>, save_path: &str) -> Result<(), Box<dyn std::error::Error>>{
    let json = serde_json::to_string(&new_post_list).unwrap();
    let mut file = File::create(save_path).expect("Failed to create file.");
    file.write_all(json.as_bytes()).expect("Failed to write to file.");
    Ok(())

}

pub async fn get_single_post_handle(postsrc: &Postsrc) -> Result<Vec<String>, Box<dyn std::error::Error>>{

    match get_blog_link_from_postsrc(&postsrc).await {
        Ok(result) => {
            let (website, mut post_list) = result;
            for post in post_list.iter_mut(){
                let temp_url = website.split('/').take(3).collect::<Vec<&str>>().join("/");
                if !temp_url.ends_with('/'){
                    *post = temp_url + "/" + post;
                }else {
                    *post = temp_url + post;
                }
            };
            Ok(post_list)
        }
        Err(error) => {
            eprintln!("Error: {}",error);
            return Err(error)
        }
    }
    
}
pub fn check_name(postname: String) -> Option<Postsrc>{
    let mut flag =false;
    for postsrc in crate::POSTSRC_LIST.iter() {
        if postname == postsrc.name {
            let name = &postsrc.name;
            let postsrc_id = &postsrc.postsrc_id;
            let link_class = &postsrc.link_class;
            let website = &postsrc.website;
            let author_class = &postsrc.author_class;
            let content_class = &postsrc.content_class;
            let time_class = &postsrc.time_class;
            let title_class = &postsrc.title_class;
            let result :Postsrc = Postsrc{name: name.to_string(), postsrc_id: postsrc.postsrc_id, website: website.to_string(), link_class: link_class.to_string(), author_class: author_class.to_string(), content_class: content_class.to_string(), title_class: title_class.to_string(), time_class: time_class.to_string()};
            return Some(result);
        } 
    }
    None
}
