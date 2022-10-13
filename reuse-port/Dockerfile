FROM golang:1.19.2 as builder

WORKDIR /app
ENV GOPROXY="https://goproxy.cn,direct"
COPY go.mod .
RUN go mod download

COPY . .

RUN go build .

ENTRYPOINT ["/app/reuse-port"]