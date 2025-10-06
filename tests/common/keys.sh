#!/bin/bash
# This script contains common signing key utilities for Atlas demos and examples

generate_signing_keys() {
    openssl genpkey -algorithm RSA -out $SIGNING_KEY -pkeyopt rsa_keygen_bits:4096 2>/dev/null
    openssl rsa -pubout -in $SIGNING_KEY -out $VERIFYING_KEY 2>/dev/null
}

remove_signing_keys() {
    rm -f *.pem
}
