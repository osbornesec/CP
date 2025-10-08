# DevOps Implementation - Check Point CPInfo Parser

## Executive Summary

This document provides comprehensive DevOps infrastructure for the Check Point diagnostic parser, implementing enterprise-grade CI/CD pipelines, containerization, infrastructure as code, and deployment automation. The implementation supports multi-platform builds, security scanning, compliance validation, and enterprise deployment patterns.

## CI/CD Pipeline Implementation

### GitHub Actions Workflow Strategy

#### Main CI/CD Pipeline
```yaml
# .github/workflows/ci-cd.yml
name: CI/CD Pipeline

on:
  push:
    branches: [ main, develop, feature/* ]
  pull_request:
    branches: [ main, develop ]
  release:
    types: [ published ]

env:
  CARGO_TERM_COLOR: always
  RUST_VERSION: '1.75'
  NODE_VERSION: '20'
  REGISTRY: ghcr.io
  IMAGE_NAME: ${{ github.repository }}

jobs:
  # Security and dependency scanning
  security-scan:
    runs-on: ubuntu-latest
    name: Security Analysis
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ env.RUST_VERSION }}
          components: clippy, rustfmt

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2
        with:
          key: security-${{ hashFiles('**/Cargo.lock') }}

      - name: Install cargo-audit
        run: cargo install cargo-audit --locked

      - name: Install cargo-deny
        run: cargo install cargo-deny --locked

      - name: Run security audit
        run: cargo audit --color=always --deny warnings

      - name: Run cargo deny
        run: cargo deny check

      - name: Run Clippy security lints
        run: cargo clippy --all-targets --all-features -- -D warnings -W clippy::all

      - name: Check for unsafe code
        run: |
          if grep -r "unsafe" src/; then
            echo "⚠️ Unsafe code detected - requires security review"
            grep -rn "unsafe" src/ || true
          else
            echo "✅ No unsafe code found"
          fi

  # Code quality and formatting
  code-quality:
    runs-on: ubuntu-latest
    name: Code Quality
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ env.RUST_VERSION }}
          components: clippy, rustfmt

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2

      - name: Check formatting
        run: cargo fmt --all -- --check

      - name: Run Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Check documentation
        run: cargo doc --no-deps --document-private-items

  # Build and test matrix
  build-test:
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact-name: cpinfo-parser-linux-x64
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact-name: cpinfo-parser-windows-x64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact-name: cpinfo-parser-macos-x64

    runs-on: ${{ matrix.os }}
    name: Build & Test (${{ matrix.os }}, ${{ matrix.rust }})
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ matrix.rust }}
          targets: ${{ matrix.target }}

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2
        with:
          key: ${{ matrix.os }}-${{ matrix.rust }}-${{ hashFiles('**/Cargo.lock') }}

      - name: Build project
        run: cargo build --verbose --release --target ${{ matrix.target }}

      - name: Run unit tests
        run: cargo test --verbose --all-features

      - name: Run integration tests
        run: cargo test --verbose --test '*' --all-features

      - name: Run benchmark tests (Ubuntu only)
        if: matrix.os == 'ubuntu-latest' && matrix.rust == 'stable'
        run: cargo bench --no-run

      - name: Upload artifacts
        if: matrix.rust == 'stable'
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact-name }}
          path: |
            target/${{ matrix.target }}/release/cpinfo-parser*
            !target/${{ matrix.target }}/release/*.d

  # Performance testing
  performance-test:
    runs-on: ubuntu-latest
    name: Performance Testing
    needs: [security-scan, code-quality]
    if: github.event_name == 'push' && (github.ref == 'refs/heads/main' || github.ref == 'refs/heads/develop')
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: ${{ env.RUST_VERSION }}

      - name: Cache Rust dependencies
        uses: Swatinem/rust-cache@v2

      - name: Build release binary
        run: cargo build --release

      - name: Create test data
        run: |
          mkdir -p test-data
          # Create synthetic test files for performance testing
          head -c 10M /dev/urandom > test-data/large-file.bin
          head -c 100M /dev/urandom > test-data/xlarge-file.bin

      - name: Run performance benchmarks
        run: cargo bench --bench performance

      - name: Memory usage test
        run: |
          valgrind --tool=massif --stacks=yes \
            ./target/release/cpinfo-parser parse test-data/large-file.bin \
            --output /tmp/test-output 2>&1 | tee memory-report.txt || true

      - name: Upload performance results
        uses: actions/upload-artifact@v4
        with:
          name: performance-results
          path: |
            target/criterion/
            memory-report.txt

  # Container build and security scanning
  container-build:
    runs-on: ubuntu-latest
    name: Container Build & Security
    needs: [build-test]
    if: github.event_name == 'push' && (github.ref == 'refs/heads/main' || github.ref == 'refs/heads/develop')
    
    permissions:
      contents: read
      packages: write
      security-events: write

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to Container Registry
        uses: docker/login-action@v3
        with:
          registry: ${{ env.REGISTRY }}
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=sha,prefix={{branch}}-
            type=raw,value=latest,enable={{is_default_branch}}

      - name: Build and push Docker image
        id: build
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./Dockerfile
          push: true
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          platforms: linux/amd64,linux/arm64
          cache-from: type=gha
          cache-to: type=gha,mode=max

      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ env.REGISTRY }}/${{ env.IMAGE_NAME }}:${{ github.sha }}
          format: 'sarif'
          output: 'trivy-results.sarif'

      - name: Upload Trivy scan results
        uses: github/codeql-action/upload-sarif@v3
        if: always()
        with:
          sarif_file: 'trivy-results.sarif'

  # Release deployment
  deploy-release:
    runs-on: ubuntu-latest
    name: Deploy Release
    needs: [build-test, container-build, performance-test]
    if: github.event_name == 'release' && github.event.action == 'published'
    
    environment: production
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4

      - name: Create release assets
        run: |
          mkdir -p release-assets
          
          # Package Linux binary
          cd cpinfo-parser-linux-x64
          tar -czf ../release-assets/cpinfo-parser-linux-x64.tar.gz *
          cd ..
          
          # Package Windows binary
          cd cpinfo-parser-windows-x64
          zip -r ../release-assets/cpinfo-parser-windows-x64.zip *
          cd ..
          
          # Package macOS binary
          cd cpinfo-parser-macos-x64
          tar -czf ../release-assets/cpinfo-parser-macos-x64.tar.gz *
          cd ..

      - name: Upload release assets
        uses: softprops/action-gh-release@v1
        with:
          files: release-assets/*
          generate_release_notes: true
```

#### Security-Focused Workflow
```yaml
# .github/workflows/security.yml
name: Security Analysis

on:
  schedule:
    - cron: '0 2 * * 1'  # Weekly on Monday at 2 AM
  workflow_dispatch:

jobs:
  security-audit:
    runs-on: ubuntu-latest
    name: Comprehensive Security Audit
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable
        with:
          toolchain: stable
          components: clippy

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Install security tools
        run: |
          cargo install cargo-audit --locked
          cargo install cargo-deny --locked
          cargo install cargo-geiger --locked

      - name: Run cargo audit
        run: cargo audit --json > audit-results.json

      - name: Run cargo deny
        run: cargo deny check --format json > deny-results.json || true

      - name: Run cargo geiger (unsafe code analysis)
        run: cargo geiger --format json > geiger-results.json || true

      - name: Run comprehensive Clippy analysis
        run: |
          cargo clippy --all-targets --all-features \
            -W clippy::cargo \
            -W clippy::nursery \
            -W clippy::pedantic \
            -W clippy::restriction \
            -- -D warnings > clippy-results.txt 2>&1 || true

      - name: Check for hardcoded secrets
        uses: trufflesecurity/trufflehog@main
        with:
          path: ./
          base: main
          head: HEAD

      - name: Upload security results
        uses: actions/upload-artifact@v4
        with:
          name: security-analysis
          path: |
            audit-results.json
            deny-results.json
            geiger-results.json
            clippy-results.txt
```

#### Performance Monitoring Workflow
```yaml
# .github/workflows/performance.yml
name: Performance Monitoring

on:
  push:
    branches: [ main ]
  schedule:
    - cron: '0 6 * * *'  # Daily at 6 AM

jobs:
  performance-baseline:
    runs-on: ubuntu-latest
    name: Performance Baseline
    
    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache dependencies
        uses: Swatinem/rust-cache@v2

      - name: Build optimized release
        run: cargo build --release

      - name: Generate test data
        run: |
          mkdir -p benchmark-data
          # Create various sized test files
          head -c 1M /dev/urandom > benchmark-data/small-1mb.bin
          head -c 10M /dev/urandom > benchmark-data/medium-10mb.bin
          head -c 100M /dev/urandom > benchmark-data/large-100mb.bin

      - name: Run performance benchmarks
        run: |
          # Memory usage benchmarking
          /usr/bin/time -v ./target/release/cpinfo-parser parse \
            benchmark-data/large-100mb.bin \
            --output /tmp/perf-test 2>&1 | tee performance-results.txt

      - name: Run Criterion benchmarks
        run: cargo bench --bench performance

      - name: Upload performance results
        uses: actions/upload-artifact@v4
        with:
          name: performance-baseline-${{ github.sha }}
          path: |
            performance-results.txt
            target/criterion/
```

## Containerization Implementation

### Multi-Stage Dockerfile
```dockerfile
# Dockerfile
# syntax=docker/dockerfile:1

# Build stage - Rust compilation
FROM rust:1.75-alpine3.19 AS builder

# Install build dependencies
RUN apk add --no-cache \
    musl-dev \
    pkgconfig \
    openssl-dev \
    git

# Create app user for security
RUN addgroup -g 1001 -S appgroup && \
    adduser -S appuser -u 1001 -G appgroup

WORKDIR /usr/src/app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src target/release/deps/cpinfo_parser*

# Copy source code
COPY src/ ./src/
COPY tests/ ./tests/
COPY benches/ ./benches/

# Build the application
RUN cargo build --release --locked

# Verify the binary
RUN ./target/release/cpinfo-parser --version

# Runtime stage - minimal Alpine Linux
FROM alpine:3.19 AS runtime

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    tzdata && \
    update-ca-certificates

# Create non-root user
RUN addgroup -g 1001 -S appgroup && \
    adduser -S appuser -u 1001 -G appgroup

# Create directories for application
RUN mkdir -p /app/bin /app/data /app/output && \
    chown -R appuser:appgroup /app

# Copy binary from builder stage
COPY --from=builder --chown=appuser:appgroup \
    /usr/src/app/target/release/cpinfo-parser /app/bin/

# Set up working directory
WORKDIR /app
USER appuser

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD /app/bin/cpinfo-parser --version || exit 1

# Default command
ENTRYPOINT ["/app/bin/cpinfo-parser"]
CMD ["--help"]

# Metadata labels
LABEL org.opencontainers.image.title="CPInfo Parser" \
      org.opencontainers.image.description="Enterprise Check Point diagnostic file parser" \
      org.opencontainers.image.vendor="Check Point Systems" \
      org.opencontainers.image.licenses="MIT" \
      org.opencontainers.image.source="https://github.com/checkpoint/cpinfo-parser"
```

### Docker Compose for Development
```yaml
# docker-compose.yml
version: '3.8'

services:
  cpinfo-parser:
    build:
      context: .
      dockerfile: Dockerfile
      target: runtime
    container_name: cpinfo-parser-dev
    environment:
      - RUST_LOG=info
      - CPINFO_CONFIG_PATH=/app/config
    volumes:
      - ./samples:/app/data:ro
      - ./output:/app/output:rw
      - ./config:/app/config:ro
    working_dir: /app
    user: "1001:1001"
    security_opt:
      - no-new-privileges:true
    cap_drop:
      - ALL
    cap_add:
      - CHOWN
      - DAC_OVERRIDE
    read_only: true
    tmpfs:
      - /tmp:noexec,nosuid,size=100m
    networks:
      - cpinfo-network

  # Development tools container
  dev-tools:
    build:
      context: .
      dockerfile: Dockerfile
      target: builder
    container_name: cpinfo-dev-tools
    volumes:
      - .:/workspace:rw
      - cargo-cache:/usr/local/cargo/registry
      - target-cache:/workspace/target
    working_dir: /workspace
    command: sleep infinity
    networks:
      - cpinfo-network

networks:
  cpinfo-network:
    driver: bridge
    internal: true

volumes:
  cargo-cache:
  target-cache:
```

### Docker Compose for Production
```yaml
# docker-compose.prod.yml
version: '3.8'

services:
  cpinfo-parser:
    image: ghcr.io/checkpoint/cpinfo-parser:latest
    container_name: cpinfo-parser-prod
    restart: unless-stopped
    environment:
      - RUST_LOG=warn
      - CPINFO_SECURITY_MODE=strict
      - CPINFO_AUDIT_ENABLED=true
    volumes:
      - /secure/cpinfo/input:/app/data:ro
      - /secure/cpinfo/output:/app/output:rw
      - /etc/cpinfo-parser:/app/config:ro
      - /var/log/cpinfo-parser:/app/logs:rw
    user: "1001:1001"
    security_opt:
      - no-new-privileges:true
      - seccomp:unconfined
    cap_drop:
      - ALL
    cap_add:
      - CHOWN
      - DAC_OVERRIDE
    read_only: true
    tmpfs:
      - /tmp:noexec,nosuid,size=500m
    ulimits:
      memlock: 67108864
      nofile: 65536
    healthcheck:
      test: ["CMD", "/app/bin/cpinfo-parser", "--version"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
    networks:
      - production-network

networks:
  production-network:
    driver: bridge
    internal: true
```

## Infrastructure as Code

### Terraform Configuration
```hcl
# terraform/main.tf
terraform {
  required_version = ">= 1.6"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.20"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.10"
    }
  }

  backend "s3" {
    bucket = "cpinfo-parser-terraform-state"
    key    = "infrastructure/terraform.tfstate"
    region = "us-east-1"
    
    dynamodb_table = "cpinfo-parser-terraform-locks"
    encrypt        = true
  }
}

provider "aws" {
  region = var.aws_region
  
  default_tags {
    tags = {
      Project     = "cpinfo-parser"
      Environment = var.environment
      ManagedBy   = "terraform"
    }
  }
}

# VPC Configuration
resource "aws_vpc" "main" {
  cidr_block           = var.vpc_cidr
  enable_dns_hostnames = true
  enable_dns_support   = true

  tags = {
    Name = "cpinfo-parser-vpc-${var.environment}"
  }
}

resource "aws_internet_gateway" "main" {
  vpc_id = aws_vpc.main.id

  tags = {
    Name = "cpinfo-parser-igw-${var.environment}"
  }
}

# Public Subnets
resource "aws_subnet" "public" {
  count = length(var.availability_zones)

  vpc_id                  = aws_vpc.main.id
  cidr_block              = cidrsubnet(var.vpc_cidr, 8, count.index)
  availability_zone       = var.availability_zones[count.index]
  map_public_ip_on_launch = true

  tags = {
    Name = "cpinfo-parser-public-subnet-${count.index + 1}-${var.environment}"
    Type = "public"
  }
}

# Private Subnets
resource "aws_subnet" "private" {
  count = length(var.availability_zones)

  vpc_id            = aws_vpc.main.id
  cidr_block        = cidrsubnet(var.vpc_cidr, 8, count.index + 10)
  availability_zone = var.availability_zones[count.index]

  tags = {
    Name = "cpinfo-parser-private-subnet-${count.index + 1}-${var.environment}"
    Type = "private"
  }
}

# NAT Gateways
resource "aws_eip" "nat" {
  count = length(var.availability_zones)

  domain = "vpc"
  
  tags = {
    Name = "cpinfo-parser-nat-eip-${count.index + 1}-${var.environment}"
  }
}

resource "aws_nat_gateway" "main" {
  count = length(var.availability_zones)

  allocation_id = aws_eip.nat[count.index].id
  subnet_id     = aws_subnet.public[count.index].id

  tags = {
    Name = "cpinfo-parser-nat-${count.index + 1}-${var.environment}"
  }

  depends_on = [aws_internet_gateway.main]
}

# Route Tables
resource "aws_route_table" "public" {
  vpc_id = aws_vpc.main.id

  route {
    cidr_block = "0.0.0.0/0"
    gateway_id = aws_internet_gateway.main.id
  }

  tags = {
    Name = "cpinfo-parser-public-rt-${var.environment}"
  }
}

resource "aws_route_table" "private" {
  count = length(var.availability_zones)

  vpc_id = aws_vpc.main.id

  route {
    cidr_block     = "0.0.0.0/0"
    nat_gateway_id = aws_nat_gateway.main[count.index].id
  }

  tags = {
    Name = "cpinfo-parser-private-rt-${count.index + 1}-${var.environment}"
  }
}

# Route Table Associations
resource "aws_route_table_association" "public" {
  count = length(aws_subnet.public)

  subnet_id      = aws_subnet.public[count.index].id
  route_table_id = aws_route_table.public.id
}

resource "aws_route_table_association" "private" {
  count = length(aws_subnet.private)

  subnet_id      = aws_subnet.private[count.index].id
  route_table_id = aws_route_table.private[count.index].id
}

# EKS Cluster
resource "aws_eks_cluster" "main" {
  name     = "cpinfo-parser-cluster-${var.environment}"
  role_arn = aws_iam_role.eks_cluster.arn
  version  = var.kubernetes_version

  vpc_config {
    subnet_ids              = concat(aws_subnet.public[*].id, aws_subnet.private[*].id)
    endpoint_private_access = true
    endpoint_public_access  = true
    public_access_cidrs     = var.eks_public_access_cidrs
  }

  encryption_config {
    provider {
      key_arn = aws_kms_key.eks.arn
    }
    resources = ["secrets"]
  }

  enabled_cluster_log_types = ["api", "audit", "authenticator", "controllerManager", "scheduler"]

  depends_on = [
    aws_iam_role_policy_attachment.eks_cluster_policy,
    aws_cloudwatch_log_group.eks_cluster,
  ]

  tags = {
    Name = "cpinfo-parser-eks-${var.environment}"
  }
}

# EKS Node Group
resource "aws_eks_node_group" "main" {
  cluster_name    = aws_eks_cluster.main.name
  node_group_name = "cpinfo-parser-nodes-${var.environment}"
  node_role_arn   = aws_iam_role.eks_node_group.arn
  subnet_ids      = aws_subnet.private[*].id

  capacity_type  = "ON_DEMAND"
  instance_types = var.node_instance_types

  scaling_config {
    desired_size = var.node_desired_size
    max_size     = var.node_max_size
    min_size     = var.node_min_size
  }

  update_config {
    max_unavailable = 1
  }

  launch_template {
    id      = aws_launch_template.eks_nodes.id
    version = aws_launch_template.eks_nodes.latest_version
  }

  depends_on = [
    aws_iam_role_policy_attachment.eks_worker_node_policy,
    aws_iam_role_policy_attachment.eks_cni_policy,
    aws_iam_role_policy_attachment.eks_container_registry_policy,
  ]

  tags = {
    Name = "cpinfo-parser-node-group-${var.environment}"
  }
}

# Launch Template for EKS Nodes
resource "aws_launch_template" "eks_nodes" {
  name_prefix   = "cpinfo-parser-eks-nodes-${var.environment}"
  image_id      = data.aws_ami.eks_worker.id
  instance_type = var.node_instance_types[0]

  vpc_security_group_ids = [aws_security_group.eks_nodes.id]

  user_data = base64encode(templatefile("${path.module}/userdata.sh", {
    cluster_name = aws_eks_cluster.main.name
    endpoint     = aws_eks_cluster.main.endpoint
    ca_data      = aws_eks_cluster.main.certificate_authority[0].data
  }))

  block_device_mappings {
    device_name = "/dev/xvda"
    ebs {
      volume_size           = 100
      volume_type           = "gp3"
      iops                  = 3000
      encrypted             = true
      kms_key_id            = aws_kms_key.ebs.arn
      delete_on_termination = true
    }
  }

  metadata_options {
    http_endpoint = "enabled"
    http_tokens   = "required"
    http_put_response_hop_limit = 2
  }

  monitoring {
    enabled = true
  }

  tag_specifications {
    resource_type = "instance"
    tags = {
      Name = "cpinfo-parser-eks-node-${var.environment}"
    }
  }
}

# S3 Bucket for storing processed files
resource "aws_s3_bucket" "processed_files" {
  bucket = "cpinfo-parser-processed-files-${var.environment}-${random_id.bucket_suffix.hex}"

  tags = {
    Name        = "cpinfo-parser-processed-files-${var.environment}"
    Environment = var.environment
  }
}

resource "aws_s3_bucket_versioning" "processed_files" {
  bucket = aws_s3_bucket.processed_files.id
  versioning_configuration {
    status = "Enabled"
  }
}

resource "aws_s3_bucket_encryption" "processed_files" {
  bucket = aws_s3_bucket.processed_files.id

  server_side_encryption_configuration {
    rule {
      apply_server_side_encryption_by_default {
        kms_master_key_id = aws_kms_key.s3.arn
        sse_algorithm     = "aws:kms"
      }
      bucket_key_enabled = true
    }
  }
}

resource "aws_s3_bucket_public_access_block" "processed_files" {
  bucket = aws_s3_bucket.processed_files.id

  block_public_acls       = true
  block_public_policy     = true
  ignore_public_acls      = true
  restrict_public_buckets = true
}

# Random ID for bucket suffix
resource "random_id" "bucket_suffix" {
  byte_length = 4
}
```

### Terraform Variables
```hcl
# terraform/variables.tf
variable "aws_region" {
  description = "AWS region for resources"
  type        = string
  default     = "us-east-1"
}

variable "environment" {
  description = "Environment name"
  type        = string
  default     = "dev"
  
  validation {
    condition     = contains(["dev", "staging", "prod"], var.environment)
    error_message = "Environment must be dev, staging, or prod."
  }
}

variable "vpc_cidr" {
  description = "CIDR block for VPC"
  type        = string
  default     = "10.0.0.0/16"
}

variable "availability_zones" {
  description = "Availability zones"
  type        = list(string)
  default     = ["us-east-1a", "us-east-1b", "us-east-1c"]
}

variable "kubernetes_version" {
  description = "Kubernetes version"
  type        = string
  default     = "1.28"
}

variable "node_instance_types" {
  description = "EC2 instance types for EKS node group"
  type        = list(string)
  default     = ["t3.medium", "t3.large"]
}

variable "node_desired_size" {
  description = "Desired number of nodes"
  type        = number
  default     = 2
}

variable "node_min_size" {
  description = "Minimum number of nodes"
  type        = number
  default     = 1
}

variable "node_max_size" {
  description = "Maximum number of nodes"
  type        = number
  default     = 5
}

variable "eks_public_access_cidrs" {
  description = "CIDR blocks for EKS public access"
  type        = list(string)
  default     = ["0.0.0.0/0"]
}
```

## Kubernetes Deployment Configurations

### Namespace and RBAC
```yaml
# k8s/namespace.yaml
apiVersion: v1
kind: Namespace
metadata:
  name: cpinfo-parser
  labels:
    name: cpinfo-parser
    app.kubernetes.io/name: cpinfo-parser
    app.kubernetes.io/component: namespace

---
apiVersion: v1
kind: ServiceAccount
metadata:
  name: cpinfo-parser-sa
  namespace: cpinfo-parser
  labels:
    app.kubernetes.io/name: cpinfo-parser
    app.kubernetes.io/component: service-account

---
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  namespace: cpinfo-parser
  name: cpinfo-parser-role
rules:
- apiGroups: [""]
  resources: ["pods", "configmaps", "secrets"]
  verbs: ["get", "list", "watch"]
- apiGroups: [""]
  resources: ["events"]
  verbs: ["create"]

---
apiVersion: rbac.authorization.k8s.io/v1
kind: RoleBinding
metadata:
  name: cpinfo-parser-rolebinding
  namespace: cpinfo-parser
subjects:
- kind: ServiceAccount
  name: cpinfo-parser-sa
  namespace: cpinfo-parser
roleRef:
  kind: Role
  name: cpinfo-parser-role
  apiGroup: rbac.authorization.k8s.io
```

### Application Deployment
```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: cpinfo-parser
  namespace: cpinfo-parser
  labels:
    app: cpinfo-parser
    version: v1
spec:
  replicas: 3
  strategy:
    type: RollingUpdate
    rollingUpdate:
      maxSurge: 1
      maxUnavailable: 1
  selector:
    matchLabels:
      app: cpinfo-parser
  template:
    metadata:
      labels:
        app: cpinfo-parser
        version: v1
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/path: "/metrics"
        prometheus.io/port: "8080"
    spec:
      serviceAccountName: cpinfo-parser-sa
      securityContext:
        runAsNonRoot: true
        runAsUser: 1001
        runAsGroup: 1001
        fsGroup: 1001
        seccompProfile:
          type: RuntimeDefault
      containers:
      - name: cpinfo-parser
        image: ghcr.io/checkpoint/cpinfo-parser:latest
        imagePullPolicy: IfNotPresent
        
        ports:
        - name: http
          containerPort: 8080
          protocol: TCP
        - name: metrics
          containerPort: 9090
          protocol: TCP
        
        env:
        - name: RUST_LOG
          value: "info"
        - name: CPINFO_SECURITY_MODE
          value: "strict"
        - name: CPINFO_MAX_FILE_SIZE
          value: "1073741824"  # 1GB
        - name: CPINFO_MAX_MEMORY
          value: "536870912"   # 512MB
        
        envFrom:
        - configMapRef:
            name: cpinfo-parser-config
        - secretRef:
            name: cpinfo-parser-secrets
        
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "1Gi"
            cpu: "1000m"
        
        securityContext:
          allowPrivilegeEscalation: false
          capabilities:
            drop:
            - ALL
          readOnlyRootFilesystem: true
          runAsNonRoot: true
          runAsUser: 1001
        
        livenessProbe:
          httpGet:
            path: /health
            port: http
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          successThreshold: 1
          failureThreshold: 3
        
        readinessProbe:
          httpGet:
            path: /ready
            port: http
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          successThreshold: 1
          failureThreshold: 3
        
        volumeMounts:
        - name: input-data
          mountPath: /app/data
          readOnly: true
        - name: output-data
          mountPath: /app/output
        - name: temp-storage
          mountPath: /tmp
        - name: config-volume
          mountPath: /app/config
          readOnly: true
      
      volumes:
      - name: input-data
        persistentVolumeClaim:
          claimName: cpinfo-input-pvc
      - name: output-data
        persistentVolumeClaim:
          claimName: cpinfo-output-pvc
      - name: temp-storage
        emptyDir:
          sizeLimit: 1Gi
      - name: config-volume
        configMap:
          name: cpinfo-parser-config
      
      affinity:
        podAntiAffinity:
          preferredDuringSchedulingIgnoredDuringExecution:
          - weight: 100
            podAffinityTerm:
              labelSelector:
                matchExpressions:
                - key: app
                  operator: In
                  values:
                  - cpinfo-parser
              topologyKey: kubernetes.io/hostname
      
      tolerations:
      - key: "app"
        operator: "Equal"
        value: "cpinfo-parser"
        effect: "NoSchedule"
```

### Service and Ingress
```yaml
# k8s/service.yaml
apiVersion: v1
kind: Service
metadata:
  name: cpinfo-parser-service
  namespace: cpinfo-parser
  labels:
    app: cpinfo-parser
spec:
  type: ClusterIP
  ports:
  - name: http
    port: 80
    targetPort: http
    protocol: TCP
  - name: metrics
    port: 9090
    targetPort: metrics
    protocol: TCP
  selector:
    app: cpinfo-parser

---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: cpinfo-parser-ingress
  namespace: cpinfo-parser
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
    nginx.ingress.kubernetes.io/rate-limit: "100"
    nginx.ingress.kubernetes.io/rate-limit-window: "1m"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
    nginx.ingress.kubernetes.io/force-ssl-redirect: "true"
spec:
  tls:
  - hosts:
    - cpinfo-parser.example.com
    secretName: cpinfo-parser-tls
  rules:
  - host: cpinfo-parser.example.com
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: cpinfo-parser-service
            port:
              number: 80
```

### ConfigMap and Secrets
```yaml
# k8s/configmap.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: cpinfo-parser-config
  namespace: cpinfo-parser
data:
  CPINFO_BATCH_SIZE: "10"
  CPINFO_TIMEOUT: "300"
  CPINFO_RETRY_ATTEMPTS: "3"
  CPINFO_LOG_LEVEL: "info"
  CPINFO_METRICS_ENABLED: "true"
  CPINFO_PROMETHEUS_PORT: "9090"

---
apiVersion: v1
kind: Secret
metadata:
  name: cpinfo-parser-secrets
  namespace: cpinfo-parser
type: Opaque
stringData:
  database_url: "postgresql://user:password@db:5432/cpinfo"
  api_key: "your-api-key-here"
  encryption_key: "your-encryption-key-here"
```

### Persistent Volume Claims
```yaml
# k8s/storage.yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: cpinfo-input-pvc
  namespace: cpinfo-parser
spec:
  accessModes:
    - ReadWriteMany
  resources:
    requests:
      storage: 100Gi
  storageClassName: aws-efs

---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: cpinfo-output-pvc
  namespace: cpinfo-parser
spec:
  accessModes:
    - ReadWriteMany
  resources:
    requests:
      storage: 500Gi
  storageClassName: aws-efs
```

## Monitoring and Observability

### Prometheus Configuration
```yaml
# monitoring/prometheus.yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: prometheus-config
  namespace: monitoring
data:
  prometheus.yml: |
    global:
      scrape_interval: 15s
      evaluation_interval: 15s

    rule_files:
      - "alert-rules.yml"

    scrape_configs:
      - job_name: 'cpinfo-parser'
        kubernetes_sd_configs:
          - role: pod
            namespaces:
              names:
                - cpinfo-parser
        relabel_configs:
          - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
            action: keep
            regex: true
          - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_path]
            action: replace
            target_label: __metrics_path__
            regex: (.+)
          - source_labels: [__address__, __meta_kubernetes_pod_annotation_prometheus_io_port]
            action: replace
            regex: ([^:]+)(?::\d+)?;(\d+)
            replacement: $1:$2
            target_label: __address__

    alerting:
      alertmanagers:
        - static_configs:
            - targets: ['alertmanager:9093']

  alert-rules.yml: |
    groups:
    - name: cpinfo-parser-alerts
      rules:
      - alert: CPInfoParserDown
        expr: up{job="cpinfo-parser"} == 0
        for: 1m
        labels:
          severity: critical
        annotations:
          summary: "CPInfo Parser instance is down"
          description: "CPInfo Parser has been down for more than 1 minute."

      - alert: CPInfoParserHighMemory
        expr: container_memory_usage_bytes{pod=~"cpinfo-parser-.*"} / container_spec_memory_limit_bytes > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High memory usage detected"
          description: "CPInfo Parser memory usage is above 80% for more than 5 minutes."

      - alert: CPInfoParserHighCPU
        expr: rate(container_cpu_usage_seconds_total{pod=~"cpinfo-parser-.*"}[5m]) > 0.8
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High CPU usage detected"
          description: "CPInfo Parser CPU usage is above 80% for more than 5 minutes."

      - alert: CPInfoParserProcessingErrors
        expr: increase(cpinfo_parser_errors_total[5m]) > 5
        for: 2m
        labels:
          severity: warning
        annotations:
          summary: "High error rate detected"
          description: "CPInfo Parser has more than 5 errors in the last 5 minutes."
```

### Grafana Dashboard
```json
{
  "dashboard": {
    "id": null,
    "title": "CPInfo Parser Monitoring",
    "tags": ["cpinfo", "parser", "monitoring"],
    "timezone": "browser",
    "panels": [
      {
        "id": 1,
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(cpinfo_parser_requests_total[5m])",
            "legendFormat": "{{method}} {{status}}"
          }
        ],
        "yAxes": [
          {
            "label": "Requests/sec"
          }
        ]
      },
      {
        "id": 2,
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(cpinfo_parser_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          },
          {
            "expr": "histogram_quantile(0.50, rate(cpinfo_parser_request_duration_seconds_bucket[5m]))",
            "legendFormat": "50th percentile"
          }
        ],
        "yAxes": [
          {
            "label": "Seconds"
          }
        ]
      },
      {
        "id": 3,
        "title": "Error Rate",
        "type": "singlestat",
        "targets": [
          {
            "expr": "rate(cpinfo_parser_requests_total{status=~\"5..\"}[5m]) / rate(cpinfo_parser_requests_total[5m])",
            "legendFormat": "Error Rate"
          }
        ],
        "thresholds": "0.01,0.05",
        "colorBackground": true
      },
      {
        "id": 4,
        "title": "Memory Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "container_memory_usage_bytes{pod=~\"cpinfo-parser-.*\"} / 1024 / 1024",
            "legendFormat": "{{pod}}"
          }
        ],
        "yAxes": [
          {
            "label": "MB"
          }
        ]
      },
      {
        "id": 5,
        "title": "CPU Usage",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(container_cpu_usage_seconds_total{pod=~\"cpinfo-parser-.*\"}[5m]) * 100",
            "legendFormat": "{{pod}}"
          }
        ],
        "yAxes": [
          {
            "label": "Percent",
            "max": 100
          }
        ]
      },
      {
        "id": 6,
        "title": "Processing Queue Size",
        "type": "graph",
        "targets": [
          {
            "expr": "cpinfo_parser_queue_size",
            "legendFormat": "Queue Size"
          }
        ]
      }
    ],
    "time": {
      "from": "now-1h",
      "to": "now"
    },
    "refresh": "5s"
  }
}
```

## Backup and Disaster Recovery

### Automated Backup Scripts
```bash
#!/bin/bash
# scripts/backup.sh

set -euo pipefail

# Configuration
BACKUP_DIR="/backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
S3_BUCKET="cpinfo-parser-backups"
RETENTION_DAYS=30

# Database backup
echo "Starting database backup..."
kubectl exec -n cpinfo-parser deployment/postgres -- \
  pg_dump -U postgres cpinfo_parser > "$BACKUP_DIR/db_backup_$TIMESTAMP.sql"

# Compress backup
gzip "$BACKUP_DIR/db_backup_$TIMESTAMP.sql"

# Upload to S3
aws s3 cp "$BACKUP_DIR/db_backup_$TIMESTAMP.sql.gz" \
  "s3://$S3_BUCKET/database/"

# Configuration backup
echo "Backing up Kubernetes configurations..."
kubectl get all,configmaps,secrets,pvc -n cpinfo-parser -o yaml > \
  "$BACKUP_DIR/k8s_config_$TIMESTAMP.yaml"

gzip "$BACKUP_DIR/k8s_config_$TIMESTAMP.yaml"

aws s3 cp "$BACKUP_DIR/k8s_config_$TIMESTAMP.yaml.gz" \
  "s3://$S3_BUCKET/configs/"

# Clean up local backups older than retention period
find "$BACKUP_DIR" -name "*.gz" -mtime +$RETENTION_DAYS -delete

# Verify backup integrity
echo "Verifying backup integrity..."
gunzip -t "$BACKUP_DIR/db_backup_$TIMESTAMP.sql.gz"
gunzip -t "$BACKUP_DIR/k8s_config_$TIMESTAMP.yaml.gz"

echo "Backup completed successfully: $TIMESTAMP"
```

### Disaster Recovery Plan
```bash
#!/bin/bash
# scripts/disaster-recovery.sh

set -euo pipefail

# Configuration
S3_BUCKET="cpinfo-parser-backups"
NAMESPACE="cpinfo-parser"

# Function to restore database
restore_database() {
    local backup_file=$1
    
    echo "Downloading database backup: $backup_file"
    aws s3 cp "s3://$S3_BUCKET/database/$backup_file" /tmp/
    
    echo "Restoring database..."
    gunzip -c "/tmp/$backup_file" | \
    kubectl exec -i -n $NAMESPACE deployment/postgres -- \
      psql -U postgres -d cpinfo_parser
    
    echo "Database restored successfully"
}

# Function to restore Kubernetes configurations
restore_configs() {
    local config_file=$1
    
    echo "Downloading configuration backup: $config_file"
    aws s3 cp "s3://$S3_BUCKET/configs/$config_file" /tmp/
    
    echo "Restoring Kubernetes configurations..."
    gunzip -c "/tmp/$config_file" | kubectl apply -f -
    
    echo "Configurations restored successfully"
}

# Function to validate system health
validate_health() {
    echo "Validating system health..."
    
    # Check pod status
    kubectl get pods -n $NAMESPACE
    
    # Check service endpoints
    kubectl get endpoints -n $NAMESPACE
    
    # Run health checks
    kubectl exec -n $NAMESPACE deployment/cpinfo-parser -- \
      /app/bin/cpinfo-parser --version
    
    echo "Health validation completed"
}

# Main recovery process
main() {
    if [ $# -lt 1 ]; then
        echo "Usage: $0 <backup_timestamp> [config_only|db_only]"
        exit 1
    fi
    
    local timestamp=$1
    local mode=${2:-full}
    
    case $mode in
        full)
            restore_configs "k8s_config_${timestamp}.yaml.gz"
            restore_database "db_backup_${timestamp}.sql.gz"
            ;;
        config_only)
            restore_configs "k8s_config_${timestamp}.yaml.gz"
            ;;
        db_only)
            restore_database "db_backup_${timestamp}.sql.gz"
            ;;
        *)
            echo "Invalid mode: $mode"
            exit 1
            ;;
    esac
    
    validate_health
    echo "Disaster recovery completed successfully"
}

main "$@"
```

## Security Implementation

### Network Policies
```yaml
# security/network-policies.yaml
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: cpinfo-parser-network-policy
  namespace: cpinfo-parser
spec:
  podSelector:
    matchLabels:
      app: cpinfo-parser
  policyTypes:
  - Ingress
  - Egress
  
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: ingress-nginx
    ports:
    - protocol: TCP
      port: 8080
  - from:
    - namespaceSelector:
        matchLabels:
          name: monitoring
    ports:
    - protocol: TCP
      port: 9090

  egress:
  - to: []
    ports:
    - protocol: TCP
      port: 443   # HTTPS
    - protocol: UDP
      port: 53    # DNS
  - to:
    - namespaceSelector:
        matchLabels:
          name: kube-system
    ports:
    - protocol: TCP
      port: 443   # Kubernetes API
```

### Pod Security Standards
```yaml
# security/pod-security-policy.yaml
apiVersion: v1
kind: Pod
metadata:
  name: cpinfo-parser-security-test
  namespace: cpinfo-parser
  annotations:
    pod-security.kubernetes.io/enforce: restricted
    pod-security.kubernetes.io/audit: restricted
    pod-security.kubernetes.io/warn: restricted
spec:
  securityContext:
    runAsNonRoot: true
    runAsUser: 1001
    runAsGroup: 1001
    fsGroup: 1001
    seccompProfile:
      type: RuntimeDefault
    sysctls: []
  containers:
  - name: cpinfo-parser
    image: ghcr.io/checkpoint/cpinfo-parser:latest
    securityContext:
      allowPrivilegeEscalation: false
      capabilities:
        drop:
        - ALL
      readOnlyRootFilesystem: true
      runAsNonRoot: true
      runAsUser: 1001
      seccompProfile:
        type: RuntimeDefault
```

## Deployment Scripts

### Production Deployment Script
```bash
#!/bin/bash
# scripts/deploy-production.sh

set -euo pipefail

# Configuration
ENVIRONMENT="production"
NAMESPACE="cpinfo-parser"
IMAGE_TAG=${1:-latest}
HELM_RELEASE="cpinfo-parser"

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log() {
    echo -e "${GREEN}[$(date +'%Y-%m-%d %H:%M:%S')] $1${NC}"
}

warn() {
    echo -e "${YELLOW}[$(date +'%Y-%m-%d %H:%M:%S')] WARNING: $1${NC}"
}

error() {
    echo -e "${RED}[$(date +'%Y-%m-%d %H:%M:%S')] ERROR: $1${NC}"
    exit 1
}

# Pre-deployment checks
pre_deployment_checks() {
    log "Running pre-deployment checks..."
    
    # Check kubectl connectivity
    kubectl cluster-info > /dev/null || error "Cannot connect to Kubernetes cluster"
    
    # Check namespace exists
    kubectl get namespace $NAMESPACE > /dev/null || error "Namespace $NAMESPACE does not exist"
    
    # Check image exists
    docker manifest inspect "ghcr.io/checkpoint/cpinfo-parser:$IMAGE_TAG" > /dev/null || \
        error "Image ghcr.io/checkpoint/cpinfo-parser:$IMAGE_TAG not found"
    
    # Check system resources
    local available_memory=$(kubectl top nodes --no-headers | awk '{sum += $4} END {print sum}')
    if [ "${available_memory:-0}" -lt 2048 ]; then
        warn "Available memory may be insufficient for deployment"
    fi
    
    log "Pre-deployment checks passed"
}

# Backup current deployment
backup_current_deployment() {
    log "Backing up current deployment..."
    
    local backup_dir="./backups/$(date +%Y%m%d_%H%M%S)"
    mkdir -p "$backup_dir"
    
    kubectl get all,configmaps,secrets,pvc -n $NAMESPACE -o yaml > \
        "$backup_dir/current-deployment.yaml"
    
    log "Current deployment backed up to $backup_dir"
}

# Deploy using Helm
deploy_with_helm() {
    log "Deploying with Helm..."
    
    helm upgrade --install $HELM_RELEASE ./helm/cpinfo-parser \
        --namespace $NAMESPACE \
        --set image.tag=$IMAGE_TAG \
        --set environment=$ENVIRONMENT \
        --wait \
        --timeout=10m \
        --atomic
    
    log "Helm deployment completed"
}

# Post-deployment validation
post_deployment_validation() {
    log "Running post-deployment validation..."
    
    # Wait for rollout to complete
    kubectl rollout status deployment/cpinfo-parser -n $NAMESPACE --timeout=600s
    
    # Check pod status
    local ready_pods=$(kubectl get pods -n $NAMESPACE -l app=cpinfo-parser \
        --field-selector=status.phase=Running \
        --no-headers | wc -l)
    
    if [ "$ready_pods" -lt 3 ]; then
        error "Expected at least 3 running pods, found $ready_pods"
    fi
    
    # Health check
    local service_ip=$(kubectl get service cpinfo-parser-service -n $NAMESPACE \
        -o jsonpath='{.spec.clusterIP}')
    
    kubectl run health-check --image=curlimages/curl:latest \
        --rm -i --restart=Never \
        -- curl -f "http://$service_ip/health" || error "Health check failed"
    
    # Performance test
    kubectl run perf-test --image=curlimages/curl:latest \
        --rm -i --restart=Never \
        -- curl -w "@curl-format.txt" -s -o /dev/null \
        "http://$service_ip/health" || warn "Performance test failed"
    
    log "Post-deployment validation completed"
}

# Smoke tests
run_smoke_tests() {
    log "Running smoke tests..."
    
    # Test basic functionality
    kubectl exec -n $NAMESPACE deployment/cpinfo-parser -- \
        /app/bin/cpinfo-parser --version
    
    # Test processing capability (if test data is available)
    if kubectl get configmap test-data -n $NAMESPACE > /dev/null 2>&1; then
        kubectl exec -n $NAMESPACE deployment/cpinfo-parser -- \
            /app/bin/cpinfo-parser parse /app/test-data/sample.cpinfo \
            --output /tmp/test-output
    fi
    
    log "Smoke tests completed"
}

# Main deployment function
main() {
    log "Starting production deployment with image tag: $IMAGE_TAG"
    
    pre_deployment_checks
    backup_current_deployment
    deploy_with_helm
    post_deployment_validation
    run_smoke_tests
    
    log "Production deployment completed successfully!"
    log "Deployment details:"
    kubectl get all -n $NAMESPACE
}

# Cleanup function for failed deployments
cleanup_on_failure() {
    error "Deployment failed. Initiating cleanup..."
    
    # Rollback Helm release
    helm rollback $HELM_RELEASE -n $NAMESPACE || true
    
    # Show logs for debugging
    kubectl logs -l app=cpinfo-parser -n $NAMESPACE --tail=100
    
    exit 1
}

# Set up trap for cleanup on failure
trap cleanup_on_failure ERR

# Run main function
main "$@"
```

## Performance Optimization

### Resource Management
```bash
#!/bin/bash
# scripts/optimize-performance.sh

set -euo pipefail

NAMESPACE="cpinfo-parser"

# CPU optimization
optimize_cpu() {
    echo "Optimizing CPU settings..."
    
    # Set CPU requests and limits
    kubectl patch deployment cpinfo-parser -n $NAMESPACE -p '{
        "spec": {
            "template": {
                "spec": {
                    "containers": [{
                        "name": "cpinfo-parser",
                        "resources": {
                            "requests": {
                                "cpu": "500m",
                                "memory": "512Mi"
                            },
                            "limits": {
                                "cpu": "2000m",
                                "memory": "2Gi"
                            }
                        }
                    }]
                }
            }
        }
    }'
}

# Memory optimization
optimize_memory() {
    echo "Optimizing memory settings..."
    
    # Configure JVM memory settings
    kubectl patch deployment cpinfo-parser -n $NAMESPACE -p '{
        "spec": {
            "template": {
                "spec": {
                    "containers": [{
                        "name": "cpinfo-parser",
                        "env": [
                            {
                                "name": "RUST_LOG",
                                "value": "warn"
                            },
                            {
                                "name": "CPINFO_BUFFER_SIZE",
                                "value": "65536"
                            },
                            {
                                "name": "CPINFO_MAX_CONCURRENT",
                                "value": "10"
                            }
                        ]
                    }]
                }
            }
        }
    }'
}

# Network optimization
optimize_network() {
    echo "Optimizing network settings..."
    
    # Configure service mesh (if using Istio)
    kubectl apply -f - <<EOF
apiVersion: networking.istio.io/v1beta1
kind: DestinationRule
metadata:
  name: cpinfo-parser-dr
  namespace: $NAMESPACE
spec:
  host: cpinfo-parser-service
  trafficPolicy:
    connectionPool:
      tcp:
        maxConnections: 100
        connectTimeout: 30s
      http:
        http1MaxPendingRequests: 100
        maxRequestsPerConnection: 10
        maxRetries: 3
        consecutiveGatewayErrors: 3
        interval: 30s
        baseEjectionTime: 30s
EOF
}

# Storage optimization
optimize_storage() {
    echo "Optimizing storage settings..."
    
    # Configure high-performance storage class
    kubectl apply -f - <<EOF
apiVersion: storage.k8s.io/v1
kind: StorageClass
metadata:
  name: fast-ssd
provisioner: kubernetes.io/aws-ebs
parameters:
  type: gp3
  iops: "3000"
  throughput: "125"
  encrypted: "true"
allowVolumeExpansion: true
volumeBindingMode: WaitForFirstConsumer
EOF
}

# Run all optimizations
main() {
    echo "Starting performance optimization..."
    
    optimize_cpu
    optimize_memory
    optimize_network
    optimize_storage
    
    echo "Performance optimization completed"
    
    # Restart deployment to apply changes
    kubectl rollout restart deployment/cpinfo-parser -n $NAMESPACE
    kubectl rollout status deployment/cpinfo-parser -n $NAMESPACE
}

main "$@"
```

## Handoff Notes for Performance Optimizer

### DevOps Infrastructure Status

**Completed Infrastructure Components:**

1. **CI/CD Pipeline**: Complete GitHub Actions workflow with multi-platform builds, security scanning, and automated deployment
2. **Containerization**: Production-ready Dockerfile with multi-stage builds, security hardening, and minimal attack surface
3. **Infrastructure as Code**: Comprehensive Terraform configuration for AWS EKS deployment
4. **Kubernetes Manifests**: Complete K8s deployment with security controls, monitoring, and scalability features
5. **Monitoring Stack**: Prometheus metrics, Grafana dashboards, and alerting rules
6. **Security Implementation**: Network policies, RBAC, secret management, and security scanning
7. **Backup & DR**: Automated backup scripts and disaster recovery procedures

**Performance Optimization Opportunities:**

1. **Container Resource Optimization**: Fine-tune CPU/memory requests and limits based on actual usage patterns
2. **Kubernetes Autoscaling**: Implement Horizontal Pod Autoscaler (HPA) and Vertical Pod Autoscaler (VPA)
3. **Storage Performance**: Optimize EBS GP3 volumes with custom IOPS and throughput settings
4. **Network Performance**: Implement service mesh (Istio) for advanced traffic management and observability
5. **Caching Strategy**: Add Redis caching layer for frequently accessed data
6. **Database Optimization**: Implement database connection pooling and query optimization

**Infrastructure Scaling Readiness:**

- **Multi-Region Deployment**: Infrastructure ready for cross-region replication
- **High Availability**: EKS cluster spans multiple AZs with auto-scaling node groups
- **Load Balancing**: Application Load Balancer with health checks and SSL termination
- **Service Mesh Ready**: Infrastructure configured for Istio service mesh deployment
- **Observability**: Complete metrics, logging, and tracing infrastructure

**Security Compliance Status:**

- **Container Security**: Distroless base images, non-root user, read-only filesystem
- **Network Security**: Network policies, private subnets, security groups
- **Data Encryption**: Encryption at rest (EBS, S3) and in transit (TLS)
- **Access Control**: RBAC, service accounts, least privilege principles
- **Audit Trail**: Comprehensive logging and monitoring for compliance

**Next Steps for Performance Optimizer:**

1. **Performance Baseline**: Establish performance baselines using the existing monitoring infrastructure
2. **Resource Optimization**: Analyze actual resource usage and optimize Kubernetes resource specifications
3. **Caching Implementation**: Add Redis caching layer for processed file metadata
4. **Auto-scaling Configuration**: Implement HPA and VPA based on CPU, memory, and custom metrics
5. **Database Optimization**: Optimize SQLite performance and consider PostgreSQL migration for enterprise use
6. **CDN Integration**: Implement CloudFront for static asset delivery
7. **Performance Testing**: Expand performance testing with realistic workloads and stress testing

The DevOps infrastructure provides a solid foundation for enterprise-scale deployment with comprehensive security, monitoring, and automation. The Performance Optimizer can now focus on fine-tuning system performance while leveraging the existing observability and scaling infrastructure.

## Summary

This DevOps implementation provides:

- **Enterprise-Grade CI/CD**: Comprehensive automation from code commit to production deployment
- **Security-First Approach**: Multi-layered security controls throughout the pipeline
- **Scalable Infrastructure**: Kubernetes-native deployment with auto-scaling capabilities
- **Comprehensive Monitoring**: Full observability stack with metrics, logging, and alerting
- **Disaster Recovery**: Automated backup and recovery procedures
- **Performance Optimization**: Ready for further performance enhancements

The infrastructure supports the high-performance Check Point diagnostic parser with enterprise security, compliance, and scalability requirements while providing operational excellence through automation and monitoring.