#!/bin/bash

# Create models directory if it doesn't exist
mkdir -p models

# Download U2Net model
echo "Downloading U2Net model..."
if command -v wget >/dev/null 2>&1; then
    wget -O models/u2net.onnx https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2net.onnx
elif command -v curl >/dev/null 2>&1; then
    curl -L https://github.com/danielgatis/rembg/releases/download/v0.0.0/u2net.onnx -o models/u2net.onnx
else
    echo "Error: Neither wget nor curl is installed. Please install one of them and try again."
    exit 1
fi

echo "Model downloaded successfully to models/u2net.onnx"

# Download Silueta model
echo "Downloading Silueta model..."
if command -v wget >/dev/null 2>&1; then
    wget -O models/silueta.onnx https://github.com/danielgatis/rembg/releases/download/v0.0.0/silueta.onnx
elif command -v curl >/dev/null 2>&1; then
    curl -L https://github.com/danielgatis/rembg/releases/download/v0.0.0/silueta.onnx -o models/silueta.onnx
else
    echo "Error: Neither wget nor curl is installed. Please install one of them and try again."
    exit 1
fi

echo "Model downloaded successfully to models/silueta.onnx"
