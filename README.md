# eMTG: A MTG Inventory Manager

eMTG is a desktop application for managing your Magic: The Gathering card collection. Built in Rust using the [egui](https://github.com/emilk/egui) framework, this tool provides a fast and intuitive way to organize, search, and update your card inventory.

## Table of Contents

- [Features](#features)
- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [License](#license)
- [Acknowledgments](#acknowledgements)

## README

This project is in its very early stages and for now it can act only as a card searcher.
The inventory part is coming next. Only tested on Linux and on the web with firefox.

## Features

- **Intuitive UI:** Enjoy a modern, easy-to-use graphical interface.
- **Inventory Management:** Add, edit, and remove cards from your collection.
- **Search & Filter:** Quickly locate specific cards using search and filtering tools.
- **Cross-Platform:** Compatible with Windows, macOS, and Linux.
- **Lightweight & Fast:** Leverages Rust’s performance for a smooth experience.

## Prerequisites

Before you begin, ensure you have met the following requirements:

- [Rust](https://www.rust-lang.org/tools/install) (version 1.81 or later)
- Cargo (Rust’s package manager)

## Installation

Clone the repository to your local machine:

    git clone https://github.com/yourusername/mtg-inventory-manager.git
    cd mtg-inventory-manager

Build the project in release mode:

    cargo build --release

## Usage

To run the application:

    cargo run --release

You can also deploy it as a WASM binary to be accessed through the web:

    trunk serve --release

Once the application is running:

- **Add Cards:** Use the provided form to input card details.
- **Edit/Remove Cards:** Update or delete entries from your inventory.
- **Search:** Utilize the search bar to filter cards by name or other attributes.


## Contributing

Contributions are welcome! If you'd like to contribute to MTG Inventory Manager, please follow these steps:

1. Fork the repository.
2. Create a new branch: `git checkout -b feature/your-feature`.
3. Commit your changes: `git commit -m 'Add some feature'`.
4. Push the branch: `git push origin feature/your-feature`.
5. Open a pull request detailing your changes.

Please ensure your code adheres to the existing style and that all tests pass.

## License

This project is licensed under the GPLv3 License.

## Acknowledgements

- **[egui](https://github.com/emilk/egui):** The modern immediate mode GUI library used to build the interface.
- **[Rust](https://www.rust-lang.org/):** The powerful and safe programming language behind this project.
- Thanks to the Magic: The Gathering community for the inspiration and ongoing support.