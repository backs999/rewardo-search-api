#!/usr/bin/env pwsh
# Script to build the Rust project for release and push the artifact to an S3 bucket
# Always uses Docker-based build with Dockerfile.s3 and S3 upload

param (
    [switch]$BuildOnly
)

# Configuration (can be overridden with environment variables)
$AWS_PROFILE = if ($env:AWS_PROFILE) { $env:AWS_PROFILE } else { "deploy-user" }
$S3_BUCKET = if ($env:S3_BUCKET) { $env:S3_BUCKET } else { "rewardo-deploy-artefacts" }
$S3_PREFIX = if ($env:S3_PREFIX) { $env:S3_PREFIX } else { "rewardo-search-api" }
$VERSION = if ($env:VERSION) { $env:VERSION } else { "latest" }
$DOCKER_IMAGE = if ($env:DOCKER_IMAGE) { $env:DOCKER_IMAGE } else { "rewardo-search-api:latest" }

# Display configuration
Write-Host "=== Build and Deploy Configuration ==="
Write-Host "AWS Profile: $AWS_PROFILE"
Write-Host "S3 Bucket: $S3_BUCKET"
Write-Host "S3 Prefix: $S3_PREFIX"
Write-Host "Version: $VERSION"
Write-Host "Build Method: Docker with Dockerfile.s3"
Write-Host "Docker Image: $DOCKER_IMAGE"
Write-Host "===================================="

# Check if Docker is installed
try {
    $dockerVersion = docker --version
    Write-Host "Docker is installed: $dockerVersion"
} catch {
    Write-Host "Error: Docker is not installed or not in PATH. Please install it first." -ForegroundColor Red
    exit 1
}

# Check AWS credentials if not in build-only mode
if (-not $BuildOnly) {
    try {
        $awsVersion = aws --version
        Write-Host "AWS CLI is installed: $awsVersion"
    } catch {
        Write-Host "Error: AWS CLI is not installed or not in PATH. Please install it first." -ForegroundColor Red
        exit 1
    }

    # Check AWS credentials
    try {
        Write-Host "Checking AWS credentials..."
        aws sts get-caller-identity --profile $AWS_PROFILE | Out-Null
        Write-Host "AWS credentials are valid." -ForegroundColor Green
    } catch {
        Write-Host "Error: AWS credentials are not configured correctly. Please run 'aws configure' or set the correct AWS_PROFILE." -ForegroundColor Red
        exit 1
    }
}

# Build Docker image using Dockerfile.s3
Write-Host "Building Docker image using Dockerfile.s3..." -ForegroundColor Cyan
try {
    docker build -t $DOCKER_IMAGE -f Dockerfile.s3 .
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Failed to build Docker image." -ForegroundColor Red
        exit 1
    }
    Write-Host "Docker image built successfully." -ForegroundColor Green
    
    if ($BuildOnly) {
        Write-Host "Build-only mode selected. Skipping S3 upload." -ForegroundColor Yellow
        Write-Host "Docker image is available as: $DOCKER_IMAGE" -ForegroundColor Green
        exit 0
    }
    
    # Run Docker container to upload to S3
    Write-Host "Running Docker container to upload to S3..." -ForegroundColor Cyan
    
    # Get AWS credentials directory path
    $awsCredentialsPath = "$env:USERPROFILE\.aws"
    if (-not (Test-Path $awsCredentialsPath)) {
        Write-Host "Warning: AWS credentials directory not found at $awsCredentialsPath" -ForegroundColor Yellow
    } else {
        Write-Host "Using AWS credentials from $awsCredentialsPath" -ForegroundColor Green
    }
    
    # Mount AWS credentials directory and pass AWS profile
    # Convert Windows path to Docker path format
    $dockerPath = $awsCredentialsPath -replace '\\', '/' -replace '^([A-Za-z]):', '//$1'
    docker run --rm -v "${dockerPath}:/root/.aws" --platform linux/amd64 -e AWS_PROFILE=$AWS_PROFILE -e S3_BUCKET=$S3_BUCKET -e S3_PREFIX=$S3_PREFIX -e VERSION=$VERSION $DOCKER_IMAGE
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Error: Failed to run Docker container for S3 upload." -ForegroundColor Red
        exit 1
    }
    Write-Host "Docker-based S3 upload completed successfully." -ForegroundColor Green
} catch {
    Write-Host "Error during Docker operations: $_" -ForegroundColor Red
    exit 1
}

Write-Host "Deployment completed successfully!" -ForegroundColor Green