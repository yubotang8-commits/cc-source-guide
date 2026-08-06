#!/usr/bin/env python3
"""Reconstruct chunked files from GitHub repo. Run from repo root after git clone."""
import os, base64, json, re

CHUNK_INFO = {
  "agents/claude-code/main.tsx": {
    "chunks": [
      "agents/claude-code/main.tsx.__part1of9__",
      "agents/claude-code/main.tsx.__part2of9__",
      "agents/claude-code/main.tsx.__part3of9__",
      "agents/claude-code/main.tsx.__part4of9__",
      "agents/claude-code/main.tsx.__part5of9__",
      "agents/claude-code/main.tsx.__part6of9__",
      "agents/claude-code/main.tsx.__part7of9__",
      "agents/claude-code/main.tsx.__part8of9__",
      "agents/claude-code/main.tsx.__part9of9__"
    ],
    "encoding": "utf-8"
  },
  "agents/claude-code/screens/REPL.tsx": {
    "chunks": [
      "agents/claude-code/screens/REPL.tsx.__part1of10__",
      "agents/claude-code/screens/REPL.tsx.__part2of10__",
      "agents/claude-code/screens/REPL.tsx.__part3of10__",
      "agents/claude-code/screens/REPL.tsx.__part4of10__",
      "agents/claude-code/screens/REPL.tsx.__part5of10__",
      "agents/claude-code/screens/REPL.tsx.__part6of10__",
      "agents/claude-code/screens/REPL.tsx.__part7of10__",
      "agents/claude-code/screens/REPL.tsx.__part8of10__",
      "agents/claude-code/screens/REPL.tsx.__part9of10__",
      "agents/claude-code/screens/REPL.tsx.__part10of10__"
    ],
    "encoding": "utf-8"
  },
  "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4": {
    "chunks": [
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part1of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part2of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part3of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part4of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part5of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part6of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part7of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part8of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part9of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part10of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part11of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part12of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part13of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part14of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part15of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part16of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part17of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part18of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part19of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part20of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part21of22__",
      "agents/opencode/artifacts/glm52-rise-video/out/glm-52-broke-out.mp4.__part22of22__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/artifacts/glm52-rise-video/public/sheep.jpg": {
    "chunks": [
      "agents/opencode/artifacts/glm52-rise-video/public/sheep.jpg.__part1of3__",
      "agents/opencode/artifacts/glm52-rise-video/public/sheep.jpg.__part2of3__",
      "agents/opencode/artifacts/glm52-rise-video/public/sheep.jpg.__part3of3__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4": {
    "chunks": [
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part1of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part2of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part3of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part4of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part5of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part6of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part7of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part8of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part9of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part10of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part11of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part12of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part13of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part14of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part15of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part16of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part17of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part18of19__",
      "agents/opencode/packages/app/src/assets/help/introducing-tabs.mp4.__part19of19__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4": {
    "chunks": [
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part1of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part2of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part3of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part4of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part5of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part6of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part7of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part8of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part9of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part10of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part11of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part12of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part13of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part14of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part15of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part16of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part17of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part18of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part19of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part20of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part21of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part22of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part23of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part24of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part25of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part26of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part27of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part28of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part29of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part30of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part31of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part32of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part33of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part34of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part35of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part36of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part37of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part38of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part39of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part40of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part41of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part42of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part43of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part44of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part45of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part46of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part47of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part48of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part49of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part50of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part51of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part52of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part53of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part54of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part55of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part56of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part57of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part58of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part59of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part60of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part61of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part62of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part63of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part64of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part65of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part66of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part67of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part68of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part69of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part70of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part71of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part72of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part73of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part74of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part75of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part76of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part77of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part78of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part79of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part80of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part81of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part82of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part83of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part84of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part85of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part86of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part87of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part88of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part89of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part90of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part91of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part92of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part93of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part94of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part95of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part96of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part97of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part98of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part99of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part100of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part101of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part102of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part103of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part104of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part105of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part106of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part107of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part108of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part109of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part110of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part111of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part112of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part113of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part114of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part115of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part116of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part117of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part118of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part119of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part120of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part121of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part122of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part123of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part124of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part125of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part126of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part127of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part128of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part129of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part130of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part131of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part132of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part133of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part134of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part135of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part136of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part137of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part138of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part139of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part140of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part141of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part142of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part143of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part144of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part145of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part146of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part147of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part148of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part149of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part150of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part151of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part152of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part153of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part154of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part155of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part156of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part157of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part158of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part159of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part160of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part161of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part162of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part163of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part164of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part165of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part166of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part167of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part168of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part169of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part170of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part171of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part172of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part173of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part174of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part175of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part176of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part177of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part178of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part179of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part180of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part181of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part182of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part183of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part184of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part185of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part186of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part187of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part188of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part189of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part190of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part191of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part192of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part193of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part194of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part195of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part196of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part197of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part198of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part199of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part200of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part201of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part202of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part203of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part204of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part205of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part206of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part207of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part208of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part209of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part210of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part211of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part212of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part213of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part214of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part215of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part216of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part217of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part218of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part219of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part220of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part221of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part222of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part223of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part224of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part225of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part226of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part227of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part228of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part229of230__",
      "agents/opencode/packages/console/app/src/asset/lander/opencode-comparison-min.mp4.__part230of230__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/console/app/src/asset/lander/screenshot-github.png": {
    "chunks": [
      "agents/opencode/packages/console/app/src/asset/lander/screenshot-github.png.__part1of4__",
      "agents/opencode/packages/console/app/src/asset/lander/screenshot-github.png.__part2of4__",
      "agents/opencode/packages/console/app/src/asset/lander/screenshot-github.png.__part3of4__",
      "agents/opencode/packages/console/app/src/asset/lander/screenshot-github.png.__part4of4__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/console/app/src/asset/lander/screenshot-splash.png": {
    "chunks": [
      "agents/opencode/packages/console/app/src/asset/lander/screenshot-splash.png.__part1of2__",
      "agents/opencode/packages/console/app/src/asset/lander/screenshot-splash.png.__part2of2__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/console/app/src/asset/lander/screenshot.png": {
    "chunks": [
      "agents/opencode/packages/console/app/src/asset/lander/screenshot.png.__part1of2__",
      "agents/opencode/packages/console/app/src/asset/lander/screenshot.png.__part2of2__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/desktop/icons/beta/icon.png": {
    "chunks": [
      "agents/opencode/packages/desktop/icons/beta/icon.png.__part1of3__",
      "agents/opencode/packages/desktop/icons/beta/icon.png.__part2of3__",
      "agents/opencode/packages/desktop/icons/beta/icon.png.__part3of3__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/desktop/icons/beta/ios/AppIcon-512@2x.png": {
    "chunks": [
      "agents/opencode/packages/desktop/icons/beta/ios/AppIcon-512@2x.png.__part1of9__",
      "agents/opencode/packages/desktop/icons/beta/ios/AppIcon-512@2x.png.__part2of9__",
      "agents/opencode/packages/desktop/icons/beta/ios/AppIcon-512@2x.png.__part3of9__",
      "agents/opencode/packages/desktop/icons/beta/ios/AppIcon-512@2x.png.__part4of9__",
      "agents/opencode/packages/desktop/icons/beta/ios/AppIcon-512@2x.png.__part5of9__",
      "agents/opencode/packages/desktop/icons/beta/ios/AppIcon-512@2x.png.__part6of9__",
      "agents/opencode/packages/desktop/icons/beta/ios/AppIcon-512@2x.png.__part7of9__",
      "agents/opencode/packages/desktop/icons/beta/ios/AppIcon-512@2x.png.__part8of9__",
      "agents/opencode/packages/desktop/icons/beta/ios/AppIcon-512@2x.png.__part9of9__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/desktop/icons/dev/icon.png": {
    "chunks": [
      "agents/opencode/packages/desktop/icons/dev/icon.png.__part1of4__",
      "agents/opencode/packages/desktop/icons/dev/icon.png.__part2of4__",
      "agents/opencode/packages/desktop/icons/dev/icon.png.__part3of4__",
      "agents/opencode/packages/desktop/icons/dev/icon.png.__part4of4__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/desktop/icons/prod/ios/AppIcon-512@2x.png": {
    "chunks": [
      "agents/opencode/packages/desktop/icons/prod/ios/AppIcon-512@2x.png.__part1of10__",
      "agents/opencode/packages/desktop/icons/prod/ios/AppIcon-512@2x.png.__part2of10__",
      "agents/opencode/packages/desktop/icons/prod/ios/AppIcon-512@2x.png.__part3of10__",
      "agents/opencode/packages/desktop/icons/prod/ios/AppIcon-512@2x.png.__part4of10__",
      "agents/opencode/packages/desktop/icons/prod/ios/AppIcon-512@2x.png.__part5of10__",
      "agents/opencode/packages/desktop/icons/prod/ios/AppIcon-512@2x.png.__part6of10__",
      "agents/opencode/packages/desktop/icons/prod/ios/AppIcon-512@2x.png.__part7of10__",
      "agents/opencode/packages/desktop/icons/prod/ios/AppIcon-512@2x.png.__part8of10__",
      "agents/opencode/packages/desktop/icons/prod/ios/AppIcon-512@2x.png.__part9of10__",
      "agents/opencode/packages/desktop/icons/prod/ios/AppIcon-512@2x.png.__part10of10__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png": {
    "chunks": [
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part1of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part2of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part3of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part4of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part5of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part6of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part7of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part8of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part9of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part10of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part11of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part12of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part13of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part14of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part15of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part16of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part17of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part18of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part19of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part20of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part21of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part22of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part23of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part24of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part25of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part26of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part27of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part28of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part29of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part30of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part31of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part32of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part33of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part34of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part35of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part36of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part37of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part38of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part39of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part40of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part41of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part42of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part43of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part44of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part45of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part46of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part47of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part48of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part49of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part50of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part51of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part52of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part53of54__",
      "agents/opencode/packages/opencode/test/image/fixtures/picture-5mb-base64.png.__part54of54__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf": {
    "chunks": [
      "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf.__part1of12__",
      "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf.__part2of12__",
      "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf.__part3of12__",
      "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf.__part4of12__",
      "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf.__part5of12__",
      "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf.__part6of12__",
      "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf.__part7of12__",
      "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf.__part8of12__",
      "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf.__part9of12__",
      "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf.__part10of12__",
      "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf.__part11of12__",
      "agents/opencode/packages/ui/src/assets/fonts/Inter.ttf.__part12of12__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png": {
    "chunks": [
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part1of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part2of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part3of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part4of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part5of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part6of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part7of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part8of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part9of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part10of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part11of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part12of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part13of14__",
      "agents/opencode/packages/web/src/assets/lander/screenshot-vscode.png.__part14of14__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/web/src/assets/lander/screenshot.png": {
    "chunks": [
      "agents/opencode/packages/web/src/assets/lander/screenshot.png.__part1of7__",
      "agents/opencode/packages/web/src/assets/lander/screenshot.png.__part2of7__",
      "agents/opencode/packages/web/src/assets/lander/screenshot.png.__part3of7__",
      "agents/opencode/packages/web/src/assets/lander/screenshot.png.__part4of7__",
      "agents/opencode/packages/web/src/assets/lander/screenshot.png.__part5of7__",
      "agents/opencode/packages/web/src/assets/lander/screenshot.png.__part6of7__",
      "agents/opencode/packages/web/src/assets/lander/screenshot.png.__part7of7__"
    ],
    "encoding": "base64"
  },
  "agents/opencode/packages/web/src/assets/web/web-homepage-see-servers.png": {
    "chunks": [
      "agents/opencode/packages/web/src/assets/web/web-homepage-see-servers.png.__part1of10__",
      "agents/opencode/packages/web/src/assets/web/web-homepage-see-servers.png.__part2of10__",
      "agents/opencode/packages/web/src/assets/web/web-homepage-see-servers.png.__part3of10__",
      "agents/opencode/packages/web/src/assets/web/web-homepage-see-servers.png.__part4of10__",
      "agents/opencode/packages/web/src/assets/web/web-homepage-see-servers.png.__part5of10__",
      "agents/opencode/packages/web/src/assets/web/web-homepage-see-servers.png.__part6of10__",
      "agents/opencode/packages/web/src/assets/web/web-homepage-see-servers.png.__part7of10__",
      "agents/opencode/packages/web/src/assets/web/web-homepage-see-servers.png.__part8of10__",
      "agents/opencode/packages/web/src/assets/web/web-homepage-see-servers.png.__part9of10__",
      "agents/opencode/packages/web/src/assets/web/web-homepage-see-servers.png.__part10of10__"
    ],
    "encoding": "base64"
  }
}

def main():
    reconstructed = 0
    failed = 0
    
    for original_path, info in CHUNK_INFO.items():
        chunks = info["chunks"]
        encoding = info["encoding"]
        
        # Ensure directory exists
        os.makedirs(os.path.dirname(original_path), exist_ok=True)
        
        # Skip if file already exists and isn't a chunk
        if os.path.exists(original_path) and "__part" not in original_path:
            print(f"  SKIP (exists): {original_path}")
            continue
        
        try:
            if encoding == "utf-8":
                content = ""
                for chunk_path in chunks:
                    with open(chunk_path, 'r', encoding='utf-8') as f:
                        content += f.read()
                with open(original_path, 'w', encoding='utf-8') as f:
                    f.write(content)
            else:  # base64
                b64_content = ""
                for chunk_path in chunks:
                    with open(chunk_path, 'r', encoding='utf-8') as f:
                        b64_content += f.read()
                raw = base64.b64decode(b64_content)
                with open(original_path, 'wb') as f:
                    f.write(raw)
            
            # Remove chunk files after reconstruction
            for chunk_path in chunks:
                if chunk_path != original_path and os.path.exists(chunk_path):
                    os.unlink(chunk_path)
            
            reconstructed += 1
            print(f"  OK: {original_path} ({len(chunks)} chunks, {encoding})")
        except Exception as e:
            failed += 1
            print(f"  FAIL: {original_path} - {e}")
    
    # Clean up empty directories left by removed chunks
    for root, dirs, files in os.walk('agents', topdown=False):
        for d in dirs:
            dp = os.path.join(root, d)
            if not os.listdir(dp):
                os.rmdir(dp)
    
    print(f"
Reconstructed: {reconstructed}, Failed: {failed}")
    if failed == 0 and reconstructed > 0:
        print("All chunked files successfully reconstructed!")

if __name__ == "__main__":
    main()
