package main

import (
	"fmt"
	"github.com/libp2p/go-reuseport"
	"io"
	"log"
	"net"
	"os"
	"os/signal"
	"syscall"
	"time"
)

func init() {
	log.SetFlags(log.Ltime | log.Lshortfile)
}

func main() {
	log.Println("start.")
	// get args from cli
	command := os.Args[1]
	if len(os.Args) < 3 {
		log.Fatalln("usage: ./cli server [port]")
	}
	port := os.Args[2]
	switch command {
	case "server":
		log.Println("normal server")
		s := Server{}
		s.start(port)
	case "reuse-port-server":
		log.Println("reuse port server")
		if len(os.Args) != 4 {
			log.Fatalln("usage: ./cli server [port]")
		}
		remotePort := os.Args[3]
		s := ReuseServer{
			localPort: port,
		}
		s.start(remotePort)
	default:
		log.Fatalln("unknown command")
	}
}

type ReuseServer struct {
	localPort string
}

func (s *ReuseServer) start(remotePort string) {

	ch := make(chan os.Signal, 1)
	signal.Notify(ch, os.Interrupt, syscall.SIGTERM)

	log.Println("listen on ", s.localPort)
	localAddr := fmt.Sprintf("192.168.1.6:%s", s.localPort)
	listen, err := reuseport.Listen("tcp", localAddr)
	if err != nil {
		log.Fatalf("listen failed %s", err)
	}
	go func() {
		tk := time.NewTicker(time.Second)
		for {
			select {
			case <-tk.C:
				remoteAddr := fmt.Sprintf("127.0.0.1:%s", remotePort)
				dial, err := reuseport.Dial("tcp", localAddr, remoteAddr)
				if err != nil {
					log.Printf("dial remote %s failed %s", remoteAddr, err)
					continue
				}
				log.Printf("dial remote %s", remoteAddr)
				if _, err := dial.Write([]byte(time.Now().String())); err != nil {
					log.Printf("dial wrtie %s\n", err)
				}
				dial.Close()
			}
		}

	}()

	for {
		select {
		case <-ch:
			return
		default:

		}
		accept, err := listen.Accept()
		if err != nil {
			log.Printf("accept failed %s", err)
			continue
		}
		all, err := io.ReadAll(accept)
		if err != nil {
			log.Printf("read failed %s", err)
			continue
		}
		log.Printf("read %s\n", string(all))
	}

}

type Server struct {
}

func (s *Server) start(port string) {

	listen, err := net.Listen("tcp", fmt.Sprintf("127.0.0.1:%s", port))
	if err != nil {
		log.Fatalf("listen failed %s", err)
	}
	for {
		accept, err := listen.Accept()
		if err != nil {
			log.Printf("accept failed %s", err)
			continue
		}
		all, err := io.ReadAll(accept)
		if err != nil {
			log.Printf("read failed %s", err)
			continue
		}
		log.Printf("remote address: %s, got: %s\n", accept.RemoteAddr().String(), string(all))
	}

}
