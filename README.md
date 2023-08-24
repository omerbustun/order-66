order-66
============

**order-66** is a command-line utility that allows users to schedule file deletions and manage them via a Telegram bot. Developed in Rust, this tool provides a simple interface to automate file deletions after a specified time.

Features
--------

*   Schedule file deletions with ease.
*   Specify the time (in minutes) after which a file will be deleted.
*   Integration with Telegram bot for management.

Installation & Usage
--------------------

To install and use **order-66**, follow these steps:

1.  Clone the repository.

    ```bash
    git clone https://github.com/omerbustun/order-66.git
    ```
2.  Navigate to the project directory and build.

    ```bash
    cd order-66
    ```

    ```bash
    cargo build --release
    ```
3.  Use the command-line utility with the following syntax:


    ```bash
    order-66 --file_path <path_to_file> --time_in_minutes <minutes>
    ```

Replace `<path_to_file>` with the full path to the file you wish to delete and `<minutes>` with the time in minutes after which the file will be deleted.

Contributing
------------

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

License
-------

This project is licensed under the GNU General Public License, version 3 (GPLv3). See the [LICENSE](LICENSE) file for the full license text.