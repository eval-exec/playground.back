package main

import (
	"github.com/libp2p/go-reuseport"
	"log"
	"net"
	"os"
	"os/signal"
	"strings"
	"syscall"
	"time"
)

func init() {
	log.SetFlags(log.Ltime | log.Lshortfile)
}

func main() {
	log.Println("start.", strings.Join(os.Args, ","))
	// get args from cli
	command := os.Args[1]
	if len(os.Args) < 3 {
		log.Fatalln("usage: ./cli server [port]")
	}
	localAddr := os.Args[2]
	switch command {
	case "server":
		log.Println("normal server")
		s := Server{
			localAddr: localAddr,
		}
		s.start()
	case "reuse-port-server":
		log.Println("reuse port server")
		remoteAddr := ""
		if len(os.Args) == 4 {
			remoteAddr = os.Args[3]
		}

		s := ReuseServer{
			localAddr: localAddr,
		}
		s.start(remoteAddr)
	default:
		log.Fatalln("unknown command", command)
	}
}

type ReuseServer struct {
	localAddr string
}

func (s *ReuseServer) start(remoteAddr string) {

	ch := make(chan os.Signal, 1)
	signal.Notify(ch, os.Interrupt, syscall.SIGTERM)

	localAddr := s.localAddr
	log.Println("node1 listen and reuse port on ", localAddr)
	listen, err := reuseport.Listen("tcp", localAddr)
	if err != nil {
		log.Fatalf("listen failed %s", err)
	}
	go func() {
		tk := time.NewTicker(time.Second)
		for {
			select {
			case <-tk.C:
				if len(remoteAddr) == 0 {
					continue
				}
				log.Printf("dialing remote %s, local: %s\n", remoteAddr, localAddr)
				dial, err := reuseport.Dial("tcp", localAddr, remoteAddr)
				if err != nil {
					log.Printf("dial remote %s failed: %s", remoteAddr, err)
					continue
				}
				log.Printf("node1: dial remote %s; local client:%s", remoteAddr, dial.LocalAddr())
				dial.Close()
			}
		}

	}()

	go func() {
		for {

			accept, err := listen.Accept()
			if err != nil {
				log.Printf("accept failed %s", err)
				continue
			}
			log.Printf("get request from %s\n", accept.RemoteAddr())
			remoteAddr = accept.RemoteAddr().String()
		}
	}()

	for {
		select {
		case s := <-ch:
			log.Println("received a signal", s)
			return
		default:

		}
	}

}

type Server struct {
	localAddr string
}

func (s *Server) start() {
	localAddr := s.localAddr

	log.Println("listening on ", localAddr)
	listen, err := net.Listen("tcp", localAddr)
	if err != nil {
		log.Fatalf("listen failed %s", err)
	}
	for {
		accept, err := listen.Accept()
		if err != nil {
			log.Printf("accept failed %s", err)
			continue
		}
		remoteAddr := accept.RemoteAddr().String()
		accept.Close()

		log.Printf("node0: get request from remote address: %s\n", remoteAddr)
		dial, err := net.Dial("tcp", remoteAddr)
		if err != nil {
			log.Printf("dial remote %s, failed %s\n", remoteAddr, err)
			continue
		}
		log.Printf("dial remote %s success\n", remoteAddr)
		dial.Close()
	}

}
