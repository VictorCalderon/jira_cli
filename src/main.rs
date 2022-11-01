use std::rc::Rc;

mod models;

mod db;
use anyhow::Context;
use db::*;

mod ui;

mod io_utils;
use io_utils::*;

mod navigator;
use navigator::*;

fn main() {
    // Get database
    let db = Rc::new(JiraDatabase::new("./data/db.json".to_owned()));

    // Instanciate navigator and get current page
    let mut navigator = Navigator::new(Rc::clone(&db));

    loop {
        // Clear the screen on start
        clearscreen::clear().unwrap();

        // Current page
        if let Some(page) = navigator.get_current_page() {
            if let Err(error) = page.draw_page() {
                println!(
                    "Error rendering page: {}\n
                    Press any key to continue...",
                    error
                );
                wait_for_key_press();
            }

            // Get user input
            let user_input = get_user_input();

            // Handle user input
            match page.handle_input(user_input.trim()) {
                Err(error) => {
                    println!(
                        "Error getting user input: {}\n
                        Press any key to continue...",
                        error
                    );
                    wait_for_key_press();
                }
                Ok(action) => {
                    if let Some(action) = action {
                        if let Err(error) = navigator.handle_action(action) {
                            println!(
                                "Error handling processing user input: {}\n
                                Press any key to continue...",
                                error
                            );
                            wait_for_key_press();
                        }
                    }
                }
            }
        }

        // // Get navigator current page
        // let page = navigator.get_current_page();

        // // Check if page Some variant came back from operation, and if not, print error and exit
        // if let Some(page) = page {
        //     if let Err(error) = page.draw_page() {
        //         println!("Error: {}", error);
        //         wait_for_key_press();
        //     }
        //     // Handle user input
        //     let input = get_user_input();
        //     // Pass input to page
        //     let action = page.handle_input(&input);
        //     // Handle input
        //     if let Ok(Some(action)) = action {
        //         navigator
        //             .handle_action(action)
        //             .expect("Action failed to perform.");
        //     }
        // }
    }
}
