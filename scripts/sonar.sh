#!/bin/bash

# SonarQube scan script for Permata Gateway (Rust)
# This script runs code quality analysis and uploads to SonarQube

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Permata Gateway SonarQube Scanner ===${NC}"

# Function to check if sonar.properties exists
check_sonar_config() {
    if [[ ! -f "sonar.properties" ]]; then
        echo -e "${RED}Error: sonar.properties not found!${NC}"
        echo -e "${YELLOW}Please create sonar.properties file with your SonarQube configuration.${NC}"
        exit 1
    fi
}

# Function to run Rust code quality checks
run_rust_quality_checks() {
    echo -e "\n${YELLOW}Running Rust code quality checks...${NC}"
    
    # Format check
    echo -e "${BLUE}Checking code format...${NC}"
    if ! cargo fmt --check; then
        echo -e "${YELLOW}Warning: Code formatting issues found. Run 'cargo fmt' to fix.${NC}"
    fi
    
    # Clippy linting
    echo -e "${BLUE}Running Clippy linter...${NC}"
    cargo clippy --all-targets --all-features -- -D warnings
    
    # Run tests with coverage (if available)
    echo -e "${BLUE}Running tests...${NC}"
    cargo test
    
    echo -e "${GREEN}✓ Rust quality checks completed${NC}"
}

# Function to generate reports for SonarQube
generate_reports() {
    echo -e "\n${YELLOW}Generating reports for SonarQube...${NC}"
    
    # Create target directory if it doesn't exist
    mkdir -p target/sonar
    
    # Generate Clippy report in JSON format (if clippy supports it)
    echo -e "${BLUE}Generating Clippy report...${NC}"
    cargo clippy --all-targets --all-features --message-format=json > target/sonar/clippy-report.json 2>/dev/null || true
    
    # Generate test report (placeholder for future test reporting)
    # cargo test --message-format=json > target/sonar/test-report.json 2>/dev/null || true
    
    echo -e "${GREEN}✓ Reports generated${NC}"
}

# Function to run SonarQube scanner
run_sonar_scanner() {
    echo -e "\n${YELLOW}Running SonarQube scanner...${NC}"
    
    # Check if docker is available
    if command -v docker &> /dev/null; then
        echo -e "${BLUE}Using Docker SonarQube scanner...${NC}"
        docker run \
            --rm \
            -v "$(pwd):/usr/src" \
            -v "$(pwd)/sonar.properties:/opt/sonar-scanner/conf/sonar-scanner.properties" \
            sonarsource/sonar-scanner-cli
    else
        echo -e "${RED}Error: Docker not found!${NC}"
        echo -e "${YELLOW}Please install Docker or use sonar-scanner CLI directly.${NC}"
        echo -e "${BLUE}Alternative: Install sonar-scanner and run: sonar-scanner${NC}"
        exit 1
    fi
}

# Function to show usage
show_usage() {
    echo -e "${BLUE}Usage: ./scripts/sonar.sh [OPTION]${NC}"
    echo -e ""
    echo -e "Options:"
    echo -e "  scan          Run full SonarQube scan (default)"
    echo -e "  check         Run only Rust quality checks"
    echo -e "  reports       Generate reports only"
    echo -e "  help          Show this help message"
    echo -e ""
    echo -e "Environment Variables:"
    echo -e "  SONAR_TOKEN   SonarQube authentication token (optional)"
    echo -e ""
    echo -e "Examples:"
    echo -e "  ./scripts/sonar.sh scan           # Full scan"
    echo -e "  ./scripts/sonar.sh check          # Quality checks only"
    echo -e "  SONAR_TOKEN=xyz ./scripts/sonar.sh scan  # With custom token"
}

# Main logic
case "${1:-scan}" in
    "scan")
        check_sonar_config
        run_rust_quality_checks
        generate_reports
        run_sonar_scanner
        ;;
    "check")
        run_rust_quality_checks
        ;;
    "reports")
        generate_reports
        ;;
    "help"|"-h"|"--help")
        show_usage
        ;;
    *)
        echo -e "${RED}Unknown option: $1${NC}"
        show_usage
        exit 1
        ;;
esac

echo -e "\n${GREEN}✓ SonarQube scan completed successfully${NC}"