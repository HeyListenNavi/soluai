<div align="center">
  <a href="https://github.com/HeyListenNavi/soluai">
    <img src="https://github.com/user-attachments/assets/759b32a5-4d22-4899-bee3-da15366d8dca" alt="SoluAI logo" width="350" height="350">
  </a>

<h1>SoluAI</h1>
<p>A video surveillance web application that detects various instances of relevance through the use of a machine learning model</p>
</div>

### Usage
To run the server `Rust` and `Cargo` are needed. The model isn't included in this repository so you'll have to get your own model and store it as data/model.onnx
When the prerequisites have been installed the project can be ran with the following command:
```
cargo run
```
The server can be also compiled with the following command:
```
cargo build --release
```
There's an index.html file that can be used to test the project