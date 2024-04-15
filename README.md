Telegram Bot for Service Appointments
=========================

This project is a Telegram bot developed in Rust using the teloxide framework and a Postgres database. The bot is designed to manage user appointments for various services.

# Features

- *User Registration: The bot initiates a dialogue upon receiving the /start command, asking for the user's name.*
- *Service Selection: After registration, the bot presents a list of available services using an inline keyboard, allowing the user to select the desired service.*
- *Time Selection: Users can choose a date and time from the available slots.*
- *Appointment Confirmation: After choosing a time, the bot requests confirmation and logs the appointment in the database.*
- *Appointment Cancellation: Users can cancel their appointments, freeing up the time for other users.*

# Technologies

- Rust: The programming language used to develop the bot.
- Teloxide: A library for building Telegram bots.
- SQLx with Postgres: Used for data management and ensuring reliable information storage about appointments.
- Actix-web: Implements a webhook to enhance integration with Telegram.


