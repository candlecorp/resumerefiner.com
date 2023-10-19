deps:
  touch resume.db
  cd ui && npm install

watch:
  cd ui && npm run build && npm run dev

dev:
  wick run app-dev.wick

start:
  wick run app.wick