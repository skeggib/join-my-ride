#!/bin/sh

tmux new-session \; \
    rename-window serve \; \
    send-keys 'cd backend/; cargo watch -x run' C-m \; \
    split-window -h \; \
    send-keys 'cd frontend/; cargo make watch' C-m \; \
    new-window \; \
    rename-window test \; \
    send-keys 'cd backend/; cargo watch -x test' C-m \; \
    split-window -h \; \
    send-keys 'cd common/; cargo watch -x test' C-m