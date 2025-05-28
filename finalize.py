#!/usr/bin/env python3
# -*- coding: utf-8 -*-

"""
Script Name: finalize.py
Author: transatoshi
Date Created: 2025-05-11
Description: This script takes care of finalizing a Grin transaction via slatepack
"""


import pexpect
import time

def run_grin_wallet():
    # Start the grin-wallet finalize command
    command = "<DIR>/grin-wallet finalize"
    child = pexpect.spawn(command)

    # Expect the password prompt and send the password
    child.expect("Password:")
    child.sendline("<PASSWORD>")

    # Expect the slatepack message prompt
    child.expect("Please paste your encoded slatepack message:")

    # Read the contents of the slatepack file
    with open("/home/grin/grin-finalize/slatepack.tmp", "r") as file:
        slatepack_content = file.read()

    # Send the contents of the slatepack file
    child.sendline(slatepack_content)

    # Finalize by hitting return
    child.sendline()

    # Wait for the command to complete
    child.expect(pexpect.EOF)

    # Print the output
    print(child.before.decode('utf-8'))

if __name__ == "__main__":
    run_grin_wallet()
