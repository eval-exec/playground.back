FROM golang:1.19.2 as builder

EXPOSE 10001

WORKDIR /app
ENV GOPROXY="https://goproxy.cn,direct"
COPY go.mod .
RUN go mod download

COPY . .

RUN go build .

CMD /app/reuse-port reuse-port-server 10001 10000