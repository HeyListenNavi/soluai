<div align="center">
  <a href="https://github.com/HeyListenNavi/soluai">
    <img src="https://github.com/user-attachments/assets/2da400d5-a016-48da-aa26-4e97cebdc2ae" alt="SoluAI logo" width="350" height="350">
  </a>


<h1>SoluAI</h1>
<p>A video surveillance web application that detects various instances of relevance through the use of a machine learning model</p>
</div>

### Usage
To run the application `Rust`, `Cargo`, and `Trunk` are needed. The model isn't included in this repository so you'll have to get your own model and store it in server/data/model.onnx
When the prerequisites have been installed the project can be used with the following commands:

#### Server
```
cargo run --bin server
```
The server can be also compiled with the following command:
```
cargo build --bin server --release
```
If you don't want to run the client there's an html file that can be used to test the server in server/index.html

####  Client
```
trunk --config client/Trunk.toml serve
```
The client can be built into client/dist with the following command:
```
trunk --config client/Trunk.toml build --release
```
