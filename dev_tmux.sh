#!/bin/sh

tmux new-session \; \
    send-keys 'cargo watch -x test' C-m \; \
    split-window -h \; \
    send-keys 'cd backend/; cargo watch -C .. -x "run backend"' C-m \; \
    split-window -v \; \
    send-keys 'cd frontend/; cargo make watch' C-m