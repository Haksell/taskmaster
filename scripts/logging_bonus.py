import http.server
import socketserver
import argparse
import threading

DEFAULT_PORT = 3000
messages = []

class MyHandler(http.server.SimpleHTTPRequestHandler):
    def do_POST(self):
        print(f"Content Length: {self.headers['Content-Length']}")
        content_length = int(self.headers['Content-Length'])
        post_data = self.rfile.read(content_length).decode('utf-8')

        messages.append(post_data)
        print(post_data)

        self.send_response(200)
        self.send_header('Connection', 'keep-alive')
        self.end_headers()

    def log_message(self, format, *args):
        pass

    def do_GET(self):
        self.send_response(200)
        self.send_header('Content-type', 'text/html')
        self.end_headers()

        refresh_script = """
        <script>
            setTimeout(function(){
                location.reload();
            }, 1000);
        </script>
        """

        message_list = "<pre style='font-family: monospace;'>"
        for message in messages:
            message_list += message + "\n"
        message_list += "</pre>"

        html_page = f"""
        <!DOCTYPE html>
        <html>
        <head>
            <title>Message Log</title>
        </head>
        <body>
            <h1>Messages</h1>
            {message_list}
            {refresh_script}
        </body>
        </html>
        """

        self.wfile.write(html_page.encode('utf-8'))

class ThreadingHTTPServer(socketserver.ThreadingMixIn, http.server.HTTPServer):
    pass

if __name__ == '__main__':
    parser = argparse.ArgumentParser(description="Simple HTTP Server")
    parser.add_argument("--port", type=int, default=DEFAULT_PORT, help="Port to listen on")
    args = parser.parse_args()

    with ThreadingHTTPServer(('0.0.0.0', args.port), MyHandler) as httpd:
        print(f"Serving on port {args.port}")
        httpd.allow_reuse_address = True
        server_thread = threading.Thread(target=httpd.serve_forever)
        server_thread.daemon = True
        server_thread.start()
        try:
            server_thread.join()
        except KeyboardInterrupt:
            pass
