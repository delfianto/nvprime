#!/bin/bash
sudo install -v -Dm644 com.github.nvprime.policy /usr/share/polkit-1/actions/
sudo install -v -Dm644 50-nvprime.rules /etc/polkit-1/rules.d/
