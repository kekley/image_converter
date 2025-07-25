# Image Converter
A simple utility for converting and resizing image files of various formats written in Rust.

# Features
- The resized image is previewed in real-time, and the following scaling algorithms are supported: Nearest-Neighbor, Bilinear, Gaussian, Catmull-Rom, Mitchell, Hamming, Lanczos3.
- Image file decoding handled by the "image" crate to support a wide range of input files.
- Extremely fast resizing using both SIMD CPU instructions and parallelization with the "rayon" crate.

# Supported Formats
Currently, the program supports converting to the following common formats: Ico (windows app icon format), Png, Jpeg, Webp. 

# Example
<img width="2548" height="1388" alt="image" src="https://github.com/user-attachments/assets/be81cfba-cab7-4cd3-9d90-9174e5f24102" />
