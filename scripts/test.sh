#!/bin/bash

# Test runner script for Permata Gateway
# Usage: ./scripts/test.sh [unit|integration|all]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== Permata Gateway Test Runner ===${NC}"

# Function to run unit tests
run_unit_tests() {
    echo -e "\n${YELLOW}Running Unit Tests...${NC}"
    cargo test --test unit_tests -- --nocapture
}

# Function to run integration tests
run_integration_tests() {
    echo -e "\n${YELLOW}Running Integration Tests...${NC}"
    cargo test --test integration_tests -- --nocapture
}

# Function to run all tests
run_all_tests() {
    echo -e "\n${YELLOW}Running All Tests...${NC}"
    cargo test -- --nocapture
}

# Function to run tests by module
run_tests_by_module() {
    local module=$1
    echo -e "\n${YELLOW}Running Tests for Module: ${module}${NC}"
    case $module in
        "config")
            cargo test config_tests -- --nocapture
            ;;
        "utils")
            cargo test json_utils_tests signature_tests -- --nocapture
            ;;
        "services")
            cargo test login_tests webhook_processor token_scheduler -- --nocapture
            ;;
        "providers")
            cargo test test_logging -- --nocapture
            ;;
        *)
            echo -e "${RED}Unknown module: ${module}${NC}"
            echo -e "${BLUE}Available modules: config, utils, services, providers${NC}"
            exit 1
            ;;
    esac
}

# Main logic
case "${1:-all}" in
    "unit")
        run_unit_tests
        ;;
    "integration")
        run_integration_tests
        ;;
    "all")
        run_all_tests
        ;;
    "config"|"utils"|"services"|"providers")
        run_tests_by_module $1
        ;;
    "help"|"-h"|"--help")
        echo -e "${BLUE}Usage: ./scripts/test.sh [OPTION]${NC}"
        echo -e ""
        echo -e "Options:"
        echo -e "  all           Run all tests (default)"
        echo -e "  unit          Run only unit tests"
        echo -e "  integration   Run only integration tests"
        echo -e "  config        Run config module tests"
        echo -e "  utils         Run utils module tests"
        echo -e "  services      Run services module tests"
        echo -e "  providers     Run providers module tests"
        echo -e "  help          Show this help message"
        ;;
    *)
        echo -e "${RED}Unknown option: $1${NC}"
        echo -e "${BLUE}Use './scripts/test.sh help' for usage information${NC}"
        exit 1
        ;;
esac

echo -e "\n${GREEN}✓ Test execution completed${NC}"