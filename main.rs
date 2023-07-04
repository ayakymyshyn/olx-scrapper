use std::io;

use async_trait::async_trait;
use scraper::{Html, Selector};

#[async_trait]
trait Scrap<T> {
    async fn scrap(&self) -> Result<Vec<T>, ScrapError>;
}

struct Scrapper {
    url: String,
}

struct Listing {
    price: String,
    title: String,
}

enum ScrapError {
    RequestError(reqwest::Error),
    ParseError(String)
}

impl From<reqwest::Error> for ScrapError {
    fn from(error: reqwest::Error) -> Self {
        Self::RequestError(error)
    }
}

#[async_trait]
impl Scrap<Listing> for Scrapper {
    async fn scrap(&self) -> Result<Vec<Listing>, ScrapError> {
        let mut listings: Vec<Listing> = Vec::new();
        println!("🚧 Scrapping data for you...");
        let response = reqwest::get(&self.url).await?.text().await?;
        let fragment = Html::parse_document(&response);
        let title_selector = Selector::parse("h6").map_err(|err| ScrapError::ParseError(err.to_string()))?;
        let price_selector = Selector::parse("p[data-testid='ad-price']").map_err(|err| ScrapError::ParseError(err.to_string()))?;
        let titles = fragment.select(&title_selector);
        let prices = fragment.select(&price_selector);

        for (title, price) in titles.zip(prices) {
            listings.push(Listing { price: price.text().collect::<String>(), title: title.text().collect::<String>() });
        }

        Ok(listings)
    }
}

#[tokio::main]
async fn main() {
    let mut query = String::new();

    println!("Type in your search query 🔍");
    io::stdin().read_line(&mut query).expect("Failed to read line!");

    let olx = Scrapper {
        url: "https://www.olx.ua/uk/list/q-".to_string() + &query.trim(),
    };

    let listings = match olx.scrap().await {
        Ok(listings) => {
            println!("Success 🙌. Data has been scraped!");
            listings
        },
        Err(ScrapError::RequestError(err)) => {
            eprintln!("Network error: {}", err);
            return;
        },
        Err(ScrapError::ParseError(err)) => {
            eprintln!("Parse error: {}", err);
            return;
        }
    };

    // TODO: Generate file with scrapped data. For example CSV or JSON
    for listing in listings {
        println!("{}: {}", listing.title, listing.price)
    }
}
