use anyhow::{anyhow, Result};
use reqwest::Client;
use reqwest_cookie_store::{CookieStore, CookieStoreMutex};

#[tokio::main]
async fn main() -> Result<()> {
    let cookie_store = CookieStore::new(None);
    let cookie_store = CookieStoreMutex::new(cookie_store);
    let cookie_store = std::sync::Arc::new(cookie_store);
    let client = reqwest::ClientBuilder::new()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:121.0) Gecko/20100101 Firefox/121.0")
        .cookie_store(true)
        .cookie_provider(std::sync::Arc::clone(&cookie_store))
        .build()
        .unwrap();
    let config = config_from_file()?;
    let csrf_token = get_csrf_token(&client).await?;
    let user = User {
        _username: config._username,
        _password: config._password,
        _csrf_token: csrf_token,
    };
    login(&client, user).await?;

    let test = get_article(&client,reqwest::Url::parse("https://www.trend.sk/biznis/tesla-nestiha-drzat-krok-tronu-mladom-trhu-coskoro-ujme-novy-lider-ciny?itm_brand=trend&itm_template=listing&itm_modul=articles-rubric-list&itm_position=6")?).await?;
    println!("{test}");
    Ok(())
}

#[derive(serde::Deserialize)]
struct Config {
    _username: String,
    _password: String,
}

#[derive(serde::Serialize)]
struct User {
    _csrf_token: String,
    _username: String,
    _password: String,
}

async fn get_article(client: &Client, url: reqwest::Url) -> Result<String> {
    let domain = url.domain();
    match domain {
        Some("www.trend.sk") | Some("trend.sk") => {}
        _ => return Err(anyhow!("nie je to trend clanok: {}", url)),
    };
    let res = client.get(url).send().await?;
    Ok(res.text().await?)
}

async fn login(client: &Client, user: User) -> Result<()> {
    let x = client
        .post("https://sso.newsandmedia.sk/login")
        .form(&user)
        .send()
        .await?;
    let y = x.cookies();
    for i in y {
        println!("{:?}", i);
    }
    Ok(())
}

fn config_from_file() -> Result<Config> {
    let config = std::fs::read_to_string("config.toml").expect("ziaden subor config.toml");
    let config: Config = toml::from_str(&config)?;
    Ok(config)
}

async fn get_csrf_token(client: &Client) -> Result<String> {
    let content = client
        .get("https://sso.newsandmedia.sk/login")
        .send()
        .await?
        .text()
        .await?;

    let document = scraper::Html::parse_document(&content);

    let csrf_selector = scraper::Selector::parse(r#"input[name="_csrf_token"]"#).unwrap();

    let csrf_element = document
        .select(&csrf_selector)
        .next()
        .expect("csrf token sa nepodarilo ziskat");
    let csrf_token = csrf_element
        .value()
        .attr("value")
        .expect("nepodarilo sa ziskat hodnotu csrf tokenu");
    Ok(csrf_token.to_string())
}
