use actix_web::{web, App, HttpResponse, HttpServer, Responder, get};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};
use sqlx::{Pool, Postgres, Row};
use dotenv::dotenv;
use log::info;
use async_trait::async_trait;


/// # Rewardo Search API
///
/// This API provides endpoints for searching reward flights based on various criteria.
/// It implements a Spring Controller API that was provided in the requirements.
///
/// The main endpoint is:
/// GET /origin/{origin}/destination/{destination}/from/{from}/to/{to}
///
/// Query parameters:
/// - page-number: The page number for pagination (default: 0)
/// - page-size: The number of items per page (default: 10)
///
/// The API returns a paginated list of reward flights matching the criteria.

// Models copied from rewardo-virgin-scraper
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RewardFlightLatest {
    pub id: Option<String>,
    pub origin: String,
    pub destination: String,
    pub departure: String,
    pub carrier_code: String,
    pub scraped_at: DateTime<Utc>,
    pub award_economy: Option<AwardEconomy>,
    pub award_business: Option<AwardBusiness>,
    pub award_premium_economy: Option<AwardPremiumEconomy>,
    pub award_first: Option<AwardFirst>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AwardEconomy {
    pub id: Option<String>,
    pub cabin_points_value: Option<i32>,
    pub is_saver_award: Option<bool>,
    pub cabin_class_seat_count: Option<i32>,
    pub cabin_class_seat_count_string: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AwardBusiness {
    pub id: Option<String>,
    pub cabin_points_value: Option<i32>,
    pub is_saver_award: Option<bool>,
    pub cabin_class_seat_count: Option<i32>,
    pub cabin_class_seat_count_string: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AwardPremiumEconomy {
    pub id: Option<String>,
    pub cabin_points_value: Option<i32>,
    pub is_saver_award: Option<bool>,
    pub cabin_class_seat_count: Option<i32>,
    pub cabin_class_seat_count_string: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AwardFirst {
    pub id: Option<String>,
    pub cabin_points_value: Option<i32>,
    pub is_saver_award: Option<bool>,
    pub cabin_class_seat_count: Option<i32>,
    pub cabin_class_seat_count_string: Option<String>,
}

// Pagination response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct Page<T> {
    pub content: Vec<T>,
    pub page_number: usize,
    pub page_size: usize,
    pub total_elements: i64,
    pub total_pages: usize,
}

// Repository trait for RewardFlightLatest
#[async_trait]
pub trait RewardFlightRepository {
    async fn find_by_origin_and_destination_and_carrier_code_and_departure_between(
        &self,
        origin: &str,
        destination: &str,
        carrier_code: &str,
        from_date: NaiveDate,
        to_date: NaiveDate,
        page_number: usize,
        page_size: usize,
    ) -> Result<Page<RewardFlightLatest>, sqlx::Error>;
    
    async fn find_all_ordered_by_lowest_cabin_points_and_origin_and_destination(
        &self,
        origin: &str,
        destination: &str,
        cabin_type: &str,
        page_number: usize,
        page_size: usize,
    ) -> Result<Page<RewardFlightLatest>, sqlx::Error>;
}

// Database implementation of the repository
pub struct RewardFlightLatestRepository {
    pool: Pool<Postgres>,
}

impl RewardFlightLatestRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RewardFlightRepository for RewardFlightLatestRepository {
    async fn find_by_origin_and_destination_and_carrier_code_and_departure_between(
        &self,
        origin: &str,
        destination: &str,
        carrier_code: &str,
        from_date: NaiveDate,
        to_date: NaiveDate,
        page_number: usize,
        page_size: usize,
    ) -> Result<Page<RewardFlightLatest>, sqlx::Error> {
        // Calculate offset
        let offset = (page_number * page_size) as i64;
        
        // Get total count using query_as instead of query_scalar! macro
        let count_query = format!(
            "SELECT COUNT(*) as count 
            FROM reward_flights_latest rfl
            WHERE rfl.origin = $1 
            AND rfl.destination = $2 
            AND rfl.carrier_code = $3 
            AND rfl.departure::date BETWEEN $4 AND $5"
        );
        
        info!("Executing count SQL query: {}", &count_query);
        info!("Count query parameters: origin={}, destination={}, carrier_code={}, from_date={}, to_date={}", 
            origin, destination, carrier_code, from_date, to_date);
            
        let count_result = sqlx::query_as::<_, (i64,)>(&count_query)
            .bind(origin)
            .bind(destination)
            .bind(carrier_code)
            .bind(from_date)
            .bind(to_date)
            .fetch_one(&self.pool)
            .await;
            
        // Log the raw count SQL response
        info!("Raw Count SQL Response: {:?}", count_result);
        
        let total_count: i64 = count_result
            .map(|row| row.0)
            .unwrap_or(0);
            
        info!("Count SQL Response: Total count = {}", total_count);

        // Get paginated results using query_as instead of query! macro
        let query = format!(
            "SELECT 
                rfl.id, 
                rfl.origin, 
                rfl.destination, 
                rfl.departure, 
                rfl.carrier_code, 
                rfl.scraped_at,
                ae.id as ae_id,
                ae.cabin_points_value as ae_cabin_points_value,
                ae.is_saver_award as ae_is_saver_award,
                ae.cabin_class_seat_count as ae_cabin_class_seat_count,
                ae.cabin_class_seat_count_string as ae_cabin_class_seat_count_string,
                ab.id as ab_id,
                ab.cabin_points_value as ab_cabin_points_value,
                ab.is_saver_award as ab_is_saver_award,
                ab.cabin_class_seat_count as ab_cabin_class_seat_count,
                ab.cabin_class_seat_count_string as ab_cabin_class_seat_count_string,
                ape.id as ape_id,
                ape.cabin_points_value as ape_cabin_points_value,
                ape.is_saver_award as ape_is_saver_award,
                ape.cabin_class_seat_count as ape_cabin_class_seat_count,
                ape.cabin_class_seat_count_string as ape_cabin_class_seat_count_string,
                af.id as af_id,
                af.cabin_points_value as af_cabin_points_value,
                af.is_saver_award as af_is_saver_award,
                af.cabin_class_seat_count as af_cabin_class_seat_count,
                af.cabin_class_seat_count_string as af_cabin_class_seat_count_string
            FROM reward_flights_latest rfl
            LEFT JOIN award_economy ae ON ae.flight_id = rfl.id
            LEFT JOIN award_business ab ON ab.flight_id = rfl.id
            LEFT JOIN award_premium_economy ape ON ape.flight_id = rfl.id
            LEFT JOIN award_first af ON af.flight_id = rfl.id
            WHERE rfl.origin = $1 
            AND rfl.destination = $2 
            AND rfl.carrier_code = $3 
            AND rfl.departure::date BETWEEN $4 AND $5
            ORDER BY rfl.departure ASC
            LIMIT $6 OFFSET $7"
        );
        
        // Execute the query with all parameters
        info!("Executing SQL query: {}", &query);
        info!("Query parameters: origin={}, destination={}, carrier_code={}, from_date={}, to_date={}, limit={}, offset={}", 
            origin, destination, carrier_code, from_date, to_date, page_size, offset);
            
        let rows = sqlx::query(&query)
            .bind(origin)
            .bind(destination)
            .bind(carrier_code)
            .bind(from_date)
            .bind(to_date)
            .bind(page_size as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;
            
        info!("SQL Response: Found {} rows", rows.len());
        
        // Log the raw SQL response data
        info!("Raw SQL Response: {:?}", rows);
        
        // Log the first row in detail to debug mapping issues
        if let Some(first_row) = rows.first() {
            println!("===== DEBUG: First row details =====");
            println!("id as i32: {:?}", first_row.try_get::<Option<i32>, _>("id"));
            println!("id as i64: {:?}", first_row.try_get::<Option<i64>, _>("id"));
            println!("id as String: {:?}", first_row.try_get::<Option<String>, _>("id"));
            println!("ae_id as i32: {:?}", first_row.try_get::<Option<i32>, _>("ae_id"));
            println!("ae_id as i64: {:?}", first_row.try_get::<Option<i64>, _>("ae_id"));
            println!("ae_id as String: {:?}", first_row.try_get::<Option<String>, _>("ae_id"));
            println!("ab_id as i32: {:?}", first_row.try_get::<Option<i32>, _>("ab_id"));
            println!("ape_id as i32: {:?}", first_row.try_get::<Option<i32>, _>("ape_id"));
            println!("af_id as i32: {:?}", first_row.try_get::<Option<i32>, _>("af_id"));
            println!("===== END DEBUG =====");
        }

        // Convert rows to RewardFlightLatest objects
        let flights = rows
            .into_iter()
            .map(|row| {
                // Construct award structs from joined columns
                // Looking at the raw SQL response, we need to check if the ae_id column exists and has a value
                let award_economy = match row.try_get::<i32, _>("ae_id") {
                    Ok(id) => {
                        // If we successfully got an ID, create the award struct
                        Some(AwardEconomy {
                            id: Some(id.to_string()),
                            cabin_points_value: row.try_get::<i32, _>("ae_cabin_points_value").ok(),
                            is_saver_award: row.try_get::<bool, _>("ae_is_saver_award").ok(),
                            cabin_class_seat_count: row.try_get::<i32, _>("ae_cabin_class_seat_count").ok(),
                            cabin_class_seat_count_string: row.try_get::<String, _>("ae_cabin_class_seat_count_string").ok(),
                        })
                    },
                    Err(_) => None
                };
                
                let award_business = match row.try_get::<i32, _>("ab_id") {
                    Ok(id) => {
                        Some(AwardBusiness {
                            id: Some(id.to_string()),
                            cabin_points_value: row.try_get::<i32, _>("ab_cabin_points_value").ok(),
                            is_saver_award: row.try_get::<bool, _>("ab_is_saver_award").ok(),
                            cabin_class_seat_count: row.try_get::<i32, _>("ab_cabin_class_seat_count").ok(),
                            cabin_class_seat_count_string: row.try_get::<String, _>("ab_cabin_class_seat_count_string").ok(),
                        })
                    },
                    Err(_) => None
                };
                
                let award_premium_economy = match row.try_get::<i32, _>("ape_id") {
                    Ok(id) => {
                        Some(AwardPremiumEconomy {
                            id: Some(id.to_string()),
                            cabin_points_value: row.try_get::<i32, _>("ape_cabin_points_value").ok(),
                            is_saver_award: row.try_get::<bool, _>("ape_is_saver_award").ok(),
                            cabin_class_seat_count: row.try_get::<i32, _>("ape_cabin_class_seat_count").ok(),
                            cabin_class_seat_count_string: row.try_get::<String, _>("ape_cabin_class_seat_count_string").ok(),
                        })
                    },
                    Err(_) => None
                };
                
                let award_first = match row.try_get::<i32, _>("af_id") {
                    Ok(id) => {
                        Some(AwardFirst {
                            id: Some(id.to_string()),
                            cabin_points_value: row.try_get::<i32, _>("af_cabin_points_value").ok(),
                            is_saver_award: row.try_get::<bool, _>("af_is_saver_award").ok(),
                            cabin_class_seat_count: row.try_get::<i32, _>("af_cabin_class_seat_count").ok(),
                            cabin_class_seat_count_string: row.try_get::<String, _>("af_cabin_class_seat_count_string").ok(),
                        })
                    },
                    Err(_) => None
                };

                // Get departure date and format it properly
                let departure: Option<NaiveDate> = row.try_get("departure").ok().flatten();
                let formatted_departure = departure.map_or_else(
                    || String::new(), 
                    |date| date.format("%Y-%m-%d").to_string()
                );
                
                // Get the ID directly as i32 and convert to string
                let id = match row.try_get::<i32, _>("id") {
                    Ok(id) => Some(id.to_string()),
                    Err(_) => None
                };
                
                RewardFlightLatest {
                    id,
                    origin: row.try_get("origin").unwrap_or_default(),
                    destination: row.try_get("destination").unwrap_or_default(),
                    departure: formatted_departure,
                    carrier_code: row.try_get("carrier_code").unwrap_or_default(),
                    scraped_at: row.try_get("scraped_at").unwrap_or_else(|_| Utc::now()),
                    award_economy,
                    award_business,
                    award_premium_economy,
                    award_first,
                }
            })
            .collect();

        // Calculate total pages
        let total_pages = (total_count as f64 / page_size as f64).ceil() as usize;

        Ok(Page {
            content: flights,
            page_number,
            page_size,
            total_elements: total_count,
            total_pages,
        })
    }
    
    async fn find_all_ordered_by_lowest_cabin_points_and_origin_and_destination(
        &self,
        origin: &str,
        destination: &str,
        cabin_type: &str,
        page_number: usize,
        page_size: usize,
    ) -> Result<Page<RewardFlightLatest>, sqlx::Error> {
        // Calculate offset
        let offset = (page_number * page_size) as i64;
        
        // Get total count
        let count_query = format!(
            "SELECT COUNT(*) as count 
            FROM reward_flights_latest rfl
            LEFT JOIN award_economy ae ON ae.flight_id = rfl.id
            LEFT JOIN award_business ab ON ab.flight_id = rfl.id
            LEFT JOIN award_premium_economy ape ON ape.flight_id = rfl.id
            WHERE rfl.origin = $1 
            AND rfl.destination = $2 
            AND (
                ($3 = 'ECONOMY' AND ae.cabin_points_value IS NOT NULL AND ae.cabin_class_seat_count > 0) OR
                ($3 = 'PREMIUM_ECONOMY' AND ape.cabin_points_value IS NOT NULL AND ape.cabin_class_seat_count > 0) OR
                ($3 = 'BUSINESS' AND ab.cabin_points_value IS NOT NULL AND ab.cabin_class_seat_count > 0)
            )"
        );
        
        info!("Executing cheapest count SQL query: {}", &count_query);
        info!("Count query parameters: origin={}, destination={}, cabin_type={}", 
            origin, destination, cabin_type);
            
        let count_result = sqlx::query_as::<_, (i64,)>(&count_query)
            .bind(origin)
            .bind(destination)
            .bind(cabin_type)
            .fetch_one(&self.pool)
            .await;
            
        info!("Raw Cheapest Count SQL Response: {:?}", count_result);
        
        let total_count: i64 = count_result
            .map(|row| row.0)
            .unwrap_or(0);
            
        info!("Cheapest Count SQL Response: Total count = {}", total_count);

        // Get paginated results
        let query = format!(
            "SELECT 
                rfl.id, 
                rfl.origin, 
                rfl.destination, 
                rfl.departure, 
                rfl.carrier_code, 
                rfl.scraped_at,
                ae.id as ae_id,
                ae.cabin_points_value as ae_cabin_points_value,
                ae.is_saver_award as ae_is_saver_award,
                ae.cabin_class_seat_count as ae_cabin_class_seat_count,
                ae.cabin_class_seat_count_string as ae_cabin_class_seat_count_string,
                ab.id as ab_id,
                ab.cabin_points_value as ab_cabin_points_value,
                ab.is_saver_award as ab_is_saver_award,
                ab.cabin_class_seat_count as ab_cabin_class_seat_count,
                ab.cabin_class_seat_count_string as ab_cabin_class_seat_count_string,
                ape.id as ape_id,
                ape.cabin_points_value as ape_cabin_points_value,
                ape.is_saver_award as ape_is_saver_award,
                ape.cabin_class_seat_count as ape_cabin_class_seat_count,
                ape.cabin_class_seat_count_string as ape_cabin_class_seat_count_string,
                af.id as af_id,
                af.cabin_points_value as af_cabin_points_value,
                af.is_saver_award as af_is_saver_award,
                af.cabin_class_seat_count as af_cabin_class_seat_count,
                af.cabin_class_seat_count_string as af_cabin_class_seat_count_string
            FROM reward_flights_latest rfl
            LEFT JOIN award_economy ae ON ae.flight_id = rfl.id
            LEFT JOIN award_business ab ON ab.flight_id = rfl.id
            LEFT JOIN award_premium_economy ape ON ape.flight_id = rfl.id
            LEFT JOIN award_first af ON af.flight_id = rfl.id
            WHERE rfl.origin = $1 
            AND rfl.destination = $2 
            AND (
                ($3 = 'ECONOMY' AND ae.cabin_points_value IS NOT NULL AND ae.cabin_class_seat_count > 0) OR
                ($3 = 'PREMIUM_ECONOMY' AND ape.cabin_points_value IS NOT NULL AND ape.cabin_class_seat_count > 0) OR
                ($3 = 'BUSINESS' AND ab.cabin_points_value IS NOT NULL AND ab.cabin_class_seat_count > 0)
            )
            ORDER BY 
                CASE 
                    WHEN $3 = 'ECONOMY' THEN ae.cabin_points_value 
                    WHEN $3 = 'PREMIUM_ECONOMY' THEN ape.cabin_points_value 
                    WHEN $3 = 'BUSINESS' THEN ab.cabin_points_value 
                END ASC,
                rfl.departure ASC
            LIMIT $4 OFFSET $5"
        );
        
        info!("Executing cheapest SQL query: {}", &query);
        info!("Query parameters: origin={}, destination={}, cabin_type={}, limit={}, offset={}", 
            origin, destination, cabin_type, page_size, offset);
            
        let rows = sqlx::query(&query)
            .bind(origin)
            .bind(destination)
            .bind(cabin_type)
            .bind(page_size as i64)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;
            
        info!("Cheapest SQL Response: Found {} rows", rows.len());
        info!("Raw Cheapest SQL Response: {:?}", rows);
        
        // Convert rows to RewardFlightLatest objects (reusing the same mapping logic)
        let flights = rows
            .into_iter()
            .map(|row| {
                let award_economy = match row.try_get::<i32, _>("ae_id") {
                    Ok(id) => {
                        Some(AwardEconomy {
                            id: Some(id.to_string()),
                            cabin_points_value: row.try_get::<i32, _>("ae_cabin_points_value").ok(),
                            is_saver_award: row.try_get::<bool, _>("ae_is_saver_award").ok(),
                            cabin_class_seat_count: row.try_get::<i32, _>("ae_cabin_class_seat_count").ok(),
                            cabin_class_seat_count_string: row.try_get::<String, _>("ae_cabin_class_seat_count_string").ok(),
                        })
                    },
                    Err(_) => None
                };
                
                let award_business = match row.try_get::<i32, _>("ab_id") {
                    Ok(id) => {
                        Some(AwardBusiness {
                            id: Some(id.to_string()),
                            cabin_points_value: row.try_get::<i32, _>("ab_cabin_points_value").ok(),
                            is_saver_award: row.try_get::<bool, _>("ab_is_saver_award").ok(),
                            cabin_class_seat_count: row.try_get::<i32, _>("ab_cabin_class_seat_count").ok(),
                            cabin_class_seat_count_string: row.try_get::<String, _>("ab_cabin_class_seat_count_string").ok(),
                        })
                    },
                    Err(_) => None
                };
                
                let award_premium_economy = match row.try_get::<i32, _>("ape_id") {
                    Ok(id) => {
                        Some(AwardPremiumEconomy {
                            id: Some(id.to_string()),
                            cabin_points_value: row.try_get::<i32, _>("ape_cabin_points_value").ok(),
                            is_saver_award: row.try_get::<bool, _>("ape_is_saver_award").ok(),
                            cabin_class_seat_count: row.try_get::<i32, _>("ape_cabin_class_seat_count").ok(),
                            cabin_class_seat_count_string: row.try_get::<String, _>("ape_cabin_class_seat_count_string").ok(),
                        })
                    },
                    Err(_) => None
                };
                
                let award_first = match row.try_get::<i32, _>("af_id") {
                    Ok(id) => {
                        Some(AwardFirst {
                            id: Some(id.to_string()),
                            cabin_points_value: row.try_get::<i32, _>("af_cabin_points_value").ok(),
                            is_saver_award: row.try_get::<bool, _>("af_is_saver_award").ok(),
                            cabin_class_seat_count: row.try_get::<i32, _>("af_cabin_class_seat_count").ok(),
                            cabin_class_seat_count_string: row.try_get::<String, _>("af_cabin_class_seat_count_string").ok(),
                        })
                    },
                    Err(_) => None
                };

                let departure: Option<NaiveDate> = row.try_get("departure").ok().flatten();
                let formatted_departure = departure.map_or_else(
                    || String::new(), 
                    |date| date.format("%Y-%m-%d").to_string()
                );
                
                let id = match row.try_get::<i32, _>("id") {
                    Ok(id) => Some(id.to_string()),
                    Err(_) => None
                };
                
                RewardFlightLatest {
                    id,
                    origin: row.try_get("origin").unwrap_or_default(),
                    destination: row.try_get("destination").unwrap_or_default(),
                    departure: formatted_departure,
                    carrier_code: row.try_get("carrier_code").unwrap_or_default(),
                    scraped_at: row.try_get("scraped_at").unwrap_or_else(|_| Utc::now()),
                    award_economy,
                    award_business,
                    award_premium_economy,
                    award_first,
                }
            })
            .collect();

        // Calculate total pages
        let total_pages = (total_count as f64 / page_size as f64).ceil() as usize;

        Ok(Page {
            content: flights,
            page_number,
            page_size,
            total_elements: total_count,
            total_pages,
        })
    }
}

// Mock implementation for testing
pub struct MockRewardFlightRepository;

#[async_trait]
impl RewardFlightRepository for MockRewardFlightRepository {
    async fn find_by_origin_and_destination_and_carrier_code_and_departure_between(
        &self,
        origin: &str,
        destination: &str,
        carrier_code: &str,
        from_date: NaiveDate,
        to_date: NaiveDate,
        page_number: usize,
        page_size: usize,
    ) -> Result<Page<RewardFlightLatest>, sqlx::Error> {
        // Create some mock data
        let mut flights = Vec::new();
        
        // Generate mock flights for each day in the date range
        let mut current_date = from_date;
        while current_date <= to_date {
            let flight = RewardFlightLatest {
                id: Some(format!("mock-{}-{}-{}", origin, destination, current_date)),
                origin: origin.to_string(),
                destination: destination.to_string(),
                departure: current_date.to_string(),
                carrier_code: carrier_code.to_string(),
                scraped_at: Utc::now(),
                award_economy: Some(AwardEconomy {
                    id: Some("mock-economy-id".to_string()),
                    cabin_points_value: Some(10000),
                    is_saver_award: Some(true),
                    cabin_class_seat_count: Some(5),
                    cabin_class_seat_count_string: Some("5".to_string()),
                }),
                award_business: Some(AwardBusiness {
                    id: Some("mock-business-id".to_string()),
                    cabin_points_value: Some(30000),
                    is_saver_award: Some(false),
                    cabin_class_seat_count: Some(2),
                    cabin_class_seat_count_string: Some("2".to_string()),
                }),
                award_premium_economy: Some(AwardPremiumEconomy {
                    id: Some("mock-premium-economy-id".to_string()),
                    cabin_points_value: Some(20000),
                    is_saver_award: Some(true),
                    cabin_class_seat_count: Some(3),
                    cabin_class_seat_count_string: Some("3".to_string()),
                }),
                award_first: None,
            };
            
            flights.push(flight);
            current_date = current_date.succ_opt().unwrap_or(current_date);
        }
        
        // Calculate total elements
        let total_elements = flights.len() as i64;
        
        // Apply pagination
        let start = page_number * page_size;
        let end = std::cmp::min(start + page_size, flights.len());
        let paginated_flights = if start < flights.len() {
            flights[start..end].to_vec()
        } else {
            Vec::new()
        };
        
        // Calculate total pages
        let total_pages = (total_elements as f64 / page_size as f64).ceil() as usize;
        
        Ok(Page {
            content: paginated_flights,
            page_number,
            page_size,
            total_elements,
            total_pages,
        })
    }
    
    async fn find_all_ordered_by_lowest_cabin_points_and_origin_and_destination(
        &self,
        origin: &str,
        destination: &str,
        cabin_type: &str,
        page_number: usize,
        page_size: usize,
    ) -> Result<Page<RewardFlightLatest>, sqlx::Error> {
        // Create some mock data
        let mut flights = Vec::new();
        
        // Generate 10 mock flights with different points values
        for i in 0..10 {
            // Create different points values based on index to simulate ordering
            let economy_points = 10000 + (i * 1000);
            let premium_economy_points = 20000 + (i * 1500);
            let business_points = 30000 + (i * 2000);
            
            // Create a date for the flight (today + i days)
            let today = chrono::Local::now().date_naive();
            let flight_date = today.checked_add_days(chrono::Days::new(i as u64)).unwrap_or(today);
            
            let flight = RewardFlightLatest {
                id: Some(format!("mock-{}-{}-{}", origin, destination, i)),
                origin: origin.to_string(),
                destination: destination.to_string(),
                departure: flight_date.to_string(),
                carrier_code: "VS".to_string(),
                scraped_at: Utc::now(),
                award_economy: Some(AwardEconomy {
                    id: Some(format!("mock-economy-id-{}", i)),
                    cabin_points_value: Some(economy_points),
                    is_saver_award: Some(true),
                    cabin_class_seat_count: Some(5),
                    cabin_class_seat_count_string: Some("5".to_string()),
                }),
                award_business: Some(AwardBusiness {
                    id: Some(format!("mock-business-id-{}", i)),
                    cabin_points_value: Some(business_points),
                    is_saver_award: Some(false),
                    cabin_class_seat_count: Some(2),
                    cabin_class_seat_count_string: Some("2".to_string()),
                }),
                award_premium_economy: Some(AwardPremiumEconomy {
                    id: Some(format!("mock-premium-economy-id-{}", i)),
                    cabin_points_value: Some(premium_economy_points),
                    is_saver_award: Some(true),
                    cabin_class_seat_count: Some(3),
                    cabin_class_seat_count_string: Some("3".to_string()),
                }),
                award_first: None,
            };
            
            flights.push(flight);
        }
        
        // Sort flights based on cabin type
        flights.sort_by(|a, b| {
            let a_points = match cabin_type {
                "ECONOMY" => a.award_economy.as_ref().and_then(|award| award.cabin_points_value),
                "PREMIUM_ECONOMY" => a.award_premium_economy.as_ref().and_then(|award| award.cabin_points_value),
                "BUSINESS" => a.award_business.as_ref().and_then(|award| award.cabin_points_value),
                _ => None,
            };
            
            let b_points = match cabin_type {
                "ECONOMY" => b.award_economy.as_ref().and_then(|award| award.cabin_points_value),
                "PREMIUM_ECONOMY" => b.award_premium_economy.as_ref().and_then(|award| award.cabin_points_value),
                "BUSINESS" => b.award_business.as_ref().and_then(|award| award.cabin_points_value),
                _ => None,
            };
            
            // Sort by points (ascending) and then by departure date
            match (a_points, b_points) {
                (Some(a_val), Some(b_val)) => a_val.cmp(&b_val),
                (Some(_), None) => std::cmp::Ordering::Less,
                (None, Some(_)) => std::cmp::Ordering::Greater,
                (None, None) => a.departure.cmp(&b.departure),
            }
        });
        
        // Calculate total elements
        let total_elements = flights.len() as i64;
        
        // Apply pagination
        let start = page_number * page_size;
        let end = std::cmp::min(start + page_size, flights.len());
        let paginated_flights = if start < flights.len() {
            flights[start..end].to_vec()
        } else {
            Vec::new()
        };
        
        // Calculate total pages
        let total_pages = (total_elements as f64 / page_size as f64).ceil() as usize;
        
        Ok(Page {
            content: paginated_flights,
            page_number,
            page_size,
            total_elements,
            total_pages,
        })
    }
}

/// Handler for retrieving the latest reward flights based on search criteria
///
/// # Parameters
/// * `origin` - The origin airport code (e.g., "LHR")
/// * `destination` - The destination airport code (e.g., "JFK")
/// * `from` - The start date for the search in YYYY-MM-DD format
/// * `to` - The end date for the search in YYYY-MM-DD format
/// * `page-number` - The page number for pagination (default: 0)
/// * `page-size` - The number of items per page (default: 10)
///
/// # Returns
/// A paginated list of reward flights matching the criteria
#[get("/api/v1/airline/vs/reward-flights/origin/{origin}/destination/{destination}/from/{from}/to/{to}")]
async fn latest_reward_flights(
    path: web::Path<(String, String, String, String)>,
    query: web::Query<PageParams>,
    repo: web::Data<RewardFlightLatestRepository>,
) -> impl Responder {
    let (origin, destination, from, to) = path.into_inner();
    let page_number = query.page_number.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(10);

    // Parse dates
    let from_date = match NaiveDate::parse_from_str(&from, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => return HttpResponse::BadRequest().body("Invalid 'from' date format. Expected YYYY-MM-DD"),
    };

    let to_date = match NaiveDate::parse_from_str(&to, "%Y-%m-%d") {
        Ok(date) => date,
        Err(_) => return HttpResponse::BadRequest().body("Invalid 'to' date format. Expected YYYY-MM-DD"),
    };

    // Query the repository
    match repo.find_by_origin_and_destination_and_carrier_code_and_departure_between(
        &origin,
        &destination,
        "VS",
        from_date,
        to_date,
        page_number as usize,
        page_size as usize,
    ).await {
        Ok(page) => HttpResponse::Ok().json(page),
        Err(e) => {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch reward flights")
        }
    }
}

/// Handler for retrieving the cheapest reward flights based on origin, destination, and cabin type
///
/// # Parameters
/// * `origin` - The origin airport code (e.g., "LHR")
/// * `destination` - The destination airport code (e.g., "JFK")
/// * `cabinType` - The cabin type (ECONOMY, PREMIUM_ECONOMY, BUSINESS)
/// * `page-number` - The page number for pagination (default: 0)
/// * `page-size` - The number of items per page (default: 50)
///
/// # Returns
/// A paginated list of reward flights ordered by lowest cabin points
#[get("/api/v1/airline/vs/reward-flights/origin/{origin}/destination/{destination}/cabin/{cabin_type}/cheapest")]
async fn cheapest_reward_flights(
    path: web::Path<(String, String, String)>,
    query: web::Query<PageParams>,
    repo: web::Data<RewardFlightLatestRepository>,
) -> impl Responder {
    let (origin, destination, cabin_type_str) = path.into_inner();
    let page_number = query.page_number.unwrap_or(0);
    let page_size = query.page_size.unwrap_or(50);
    
    // Validate cabin type
    let cabin_type = match cabin_type_str.as_str() {
        "ECONOMY" | "PREMIUM_ECONOMY" | "BUSINESS" => cabin_type_str,
        _ => return HttpResponse::BadRequest().body("Invalid cabin type. Expected ECONOMY, PREMIUM_ECONOMY, or BUSINESS"),
    };

    // Query the repository
    match repo.find_all_ordered_by_lowest_cabin_points_and_origin_and_destination(
        &origin,
        &destination,
        &cabin_type,
        page_number as usize,
        page_size as usize,
    ).await {
        Ok(page) => HttpResponse::Ok().json(page),
        Err(e) => {
            log::error!("Database error: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch cheapest reward flights")
        }
    }
}

// Query parameters for pagination
#[derive(Debug, Deserialize)]
struct PageParams {
    #[serde(rename = "page-number")]
    page_number: Option<i32>,
    #[serde(rename = "page-size")]
    page_size: Option<i32>,
}

// Enum for cabin types
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum CabinType {
    Economy,
    PremiumEconomy,
    Business,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize environment
    dotenv().ok();
    env_logger::init();

    info!("Starting server at http://127.0.0.1:8080");

    // Get database URL from environment
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");
    
    // Create database connection pool and test connection
    info!("Testing database connection...");
    let pool = match sqlx::postgres::PgPool::connect(&database_url).await {
        Ok(pool) => {
            // Test the connection by executing a simple query
            match sqlx::query("SELECT 1").execute(&pool).await {
                Ok(_) => {
                    info!("Database connection successful");
                    pool
                },
                Err(e) => {
                    log::error!("Database connection test failed: {}", e);
                    panic!("Failed to connect to database: {}", e);
                }
            }
        },
        Err(e) => {
            log::error!("Failed to create database connection pool: {}", e);
            panic!("Failed to create database connection pool: {}", e);
        }
    };

    // Create repository with database connection
    let repository = web::Data::new(RewardFlightLatestRepository::new(pool));

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(repository.clone())
            .service(latest_reward_flights)
            .service(cheapest_reward_flights)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
