use reqwest::blocking::Client;
use serde::Deserialize;
use std::{collections::HashMap, f64, fs};

#[derive(Deserialize)]
struct WeatherCondition {
    text: String,
}

#[derive(Deserialize)]
struct WeatherForecastDayResult {
    condition: WeatherCondition,
    maxtemp_c: f64,
    mintemp_c: f64,
    daily_chance_of_rain: i64,
    daily_chance_of_snow: i64,
}

#[derive(Deserialize)]
struct WeatherForecastDay {
    day: WeatherForecastDayResult,
}

#[derive(Deserialize)]
struct WeatherForecast {
    #[serde(rename(deserialize = "forecastday"))]
    forecast_day: Vec<WeatherForecastDay>,
}

#[derive(Deserialize)]
struct WeatherTopResult {
    forecast: WeatherForecast,
}

#[derive(Deserialize)]
struct WeatherApiConf {
    token: String,
}

#[derive(Deserialize)]
struct DiscordUserConf {
    id: String,
    city: String,
}

#[derive(Deserialize)]
struct DiscordConf {
    webhook_url: String,
    users: Vec<DiscordUserConf>,
}

#[derive(Deserialize)]
struct Conf {
    weather_api: WeatherApiConf,
    discord: DiscordConf,
}

fn main() {
    let client = Client::new();

    let conf: Conf = toml::from_str(
        &fs::read_to_string("conf.toml").expect("couldn't read the configuration file `conf.toml`"),
    )
    .expect("couldn't parse the provided toml configuration");

    let webhook_url = &conf.discord.webhook_url;

    for user in conf.discord.users {
        let weather = get_weather(&client, &conf.weather_api.token, &user.city, false, false);

        send_message(
            &client,
            &webhook_url,
            &format!(
                "<@{}> hello there, it is {} today in {} with a max temperature of {} and min temperature of {}.\
                    The chance of rain is {} and the chance of snow is {}.
                    Have a great day!",
                user.id,
                weather.forecast.forecast_day[0].day.condition.text,
                user.city,
                weather.forecast.forecast_day[0].day.maxtemp_c,
                weather.forecast.forecast_day[0].day.mintemp_c,
                weather.forecast.forecast_day[0].day.daily_chance_of_rain,
                weather.forecast.forecast_day[0].day.daily_chance_of_snow,
            ),
        );
    }
}

fn get_weather(
    web_client: &Client,
    apikey: &str,
    city: &str,
    aqi: bool,
    alerts: bool,
) -> WeatherTopResult {
    let weather_url = format!(
        "https://api.weatherapi.com/v1/forecast.json?key={}&q={}&days=1&aqi={}&alerts={}",
        apikey,
        city,
        if aqi { "yes" } else { "no" },
        if alerts { "yes" } else { "no" },
    );

    web_client
        .get(weather_url)
        .send()
        .expect("failed to call the weather api")
        .json()
        .expect("failed to desiarlize the weather api response")
}

fn send_message(web_client: &Client, webhook: &str, message: &str) {
    let mut body = HashMap::new();
    body.insert("content", message);

    let _res = web_client
        .post(webhook)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .expect("something went wrong with the webhook call");
}
