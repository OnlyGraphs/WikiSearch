FROM node:16 AS frontend
WORKDIR /usr/app/
ARG TOKEN
ARG BACKEND
RUN git clone https://github.com/OnlyGraphs/FrontEnd.git
WORKDIR /usr/app/FrontEnd
RUN echo NEXT_PUBLIC_BACKEND=${BACKEND} >> .env.local
RUN npm install
RUN npm run build

FROM rust:latest AS backend
MAINTAINER Kyle Cotton <kylecottonkc@gmail.com>
WORKDIR /usr/src/search
COPY . .
RUN cargo build --release
COPY ./staticfiles/* ./out/*
COPY --from=frontend /usr/app/FrontEnd/out ./out

FROM gcr.io/distroless/cc-debian10
MAINTAINER Kyle Cotton <kylecottonkc@gmail.com>
COPY --from=backend /usr/src/search/target/release/search_api .
COPY --from=backend /usr/src/search/out ./out

CMD ["./search_api"]