#!/bin/bash

# Test script for 8086 disassembler
# This script:
# 1. Assembles each .asm file in tests/ with nasm
# 2. Runs the disassembler on the assembled binary
# 3. Reassembles the disassembler output
# 4. Compares the original and reassembled binaries


# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Check if nasm is installed
if ! command -v nasm &> /dev/null; then
    echo -e "${RED}Error: nasm is not installed${NC}"
    echo "Please install nasm to run these tests"
    exit 1
fi

# Build the disassembler
echo -e "${YELLOW}Building disassembler...${NC}"
cargo build --release

DISASSEMBLER="./target/release/cpu_parser"
TESTS_DIR="tests"
TEMP_DIR="test_temp"

# Create temporary directory
mkdir -p "$TEMP_DIR"

# Function to clean up temp files
cleanup() {
    rm -rf "$TEMP_DIR"
}

# Set up cleanup on script exit
trap cleanup EXIT

# Counter for test results
passed=0
failed=0

echo -e "${YELLOW}Running disassembler tests...${NC}"
echo "=================================="

# Process each .asm file in tests directory
for asm_file in "$TESTS_DIR"/*.asm; do
    if [ ! -f "$asm_file" ]; then
        echo -e "${RED}No .asm files found in $TESTS_DIR${NC}"
        exit 1
    fi
    
    filename=$(basename "$asm_file" .asm)
    echo -e "${YELLOW}Testing: $filename${NC}"
    
    # Step 1: Assemble original .asm file with nasm
    echo "  Assembling original..."
    nasm -f bin -o "$TEMP_DIR/${filename}_original.bin" "$asm_file"
    
    # Step 2: Run disassembler on the binary
    echo "  Running disassembler..."
    "$DISASSEMBLER" "$TEMP_DIR/${filename}_original.bin" > "$TEMP_DIR/${filename}_disassembled.asm" 2>/dev/null
    
    # Step 3: Assemble the disassembler output
    echo "  Reassembling disassembler output..."
    nasm -f bin -o "$TEMP_DIR/${filename}_reassembled.bin" "$TEMP_DIR/${filename}_disassembled.asm"
    
    # Step 4: Compare the binaries
    echo "  Comparing binaries..."
    if cmp -s "$TEMP_DIR/${filename}_original.bin" "$TEMP_DIR/${filename}_reassembled.bin"; then
        echo -e "  ${GREEN}✓ PASSED${NC}"
        ((passed++))
    else
        echo -e "  ${RED}✗ FAILED${NC}"
        echo "  Original size: $(wc -c < "$TEMP_DIR/${filename}_original.bin") bytes"
        echo "  Reassembled size: $(wc -c < "$TEMP_DIR/${filename}_reassembled.bin") bytes"
        
        # Show hex dump comparison for debugging
        echo "  Hex dump comparison:"
        echo "  Original:"
        hexdump -C "$TEMP_DIR/${filename}_original.bin" | head -5
        echo "  Reassembled:"
        hexdump -C "$TEMP_DIR/${filename}_reassembled.bin" | head -5
        
        ((failed++))
    fi
    
    echo ""
done

# Print summary
echo "=================================="
echo -e "${YELLOW}Test Summary:${NC}"
echo -e "${GREEN}Passed: $passed${NC}"
echo -e "${RED}Failed: $failed${NC}"
echo "Total: $((passed + failed))"

if [ $failed -eq 0 ]; then
    echo -e "${GREEN}All tests passed!${NC}"
    exit 0
else
    echo -e "${RED}Some tests failed!${NC}"
    exit 1
fi 