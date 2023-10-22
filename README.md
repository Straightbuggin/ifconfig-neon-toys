# ifconfig-neon-toys

A simple Rust web application to display client's IP address and request headers. Primarly built for funi. Inspired by ifconfig.io, but built in rust. 

## Features

- Displays the client's IP address.
- Shows the request headers sent by the client in a tabular format.
- Uses a thread pool to handle incoming connections concurrently.
- Responds with a HTML page rendered on the server-side.



## Usage

1. **Clone the repository:**
```bash
git clone https://github.com/Straightbuggin/ifconfig-neon-toys.git
cd ifconfig-neon-toys
```

2. **Build the project:**
```bash
cargo build --release
```

3. **Run the application:**
```bash
cargo run --release
```

The application will start a server on `http://[::]:8080`. Open this URL in your web browser to view your IP address and request headers.


## Docker Installation

1. **Build the Docker image:**
```bash
docker build -t ifconfig-neon-toys .
```

2. **Run the Docker container:**
```bash
docker run -d -p 8080:8080 --name ifconfig-neon-toys-container ifconfig-neon-toys
```

Now, access the application at `http://localhost:8080` or `http://[::]:8080` from your web browser to see your IP address and request headers.

## Contributing

Feel free to fork the repository, create a feature branch, and submit a pull request.
