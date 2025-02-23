"""
This module defines the `TonlineSpider` class, which is a Scrapy spider used to
crawl news articles from the t-online website (https://www.t-online.de/).

The spider starts at the given `start_urls` and processes links by adding them
to a queue. It crawls the links, processes article pages, and extracts content
only for articles that are published after a specified date threshold.

Usage:
    To run the spider, use Scrapy's command line tool with the spider name:
    `scrapy crawl tonline_spider`

Attributes:
    None: This module does not define any module-level variables.

Example:
    - Start the spider: `scrapy crawl tonline_spider`
    - It will fetch news articles from the t-online website.
"""

import scrapy
from queue import Queue
from scrapy.http import Request
from datetime import datetime, timezone

class TonlineSpider(scrapy.Spider):
    """Spider to crawl and scrape news articles from t-online's news website.

    The `TonlineSpider` follows links from the main page and article pages. It
    adds discovered links to a queue for further processing and scrapes article
    content, including the article's URL, text content, and date. The spider
    only extracts articles published after a defined date threshold.

    Attributes:
        visited_links (set): A set of visited URLs to avoid revisiting the same link.
        link_queue (Queue): A FIFO queue used to store and manage the URLs to crawl.
        date_threshold (datetime): The cutoff date; articles published before this
            date will be ignored.
    
    Methods:
        start_requests(): Starts the crawling process by enqueueing the initial URLs.
        process_links(): Processes URLs in the queue and sends requests for each.
        parse_frontpage(response): Extracts article links from the front page and adds
            them to the queue for processing.
        parse_article(response): Extracts content from individual articles and saves
            them if they meet the date threshold.
    """
    name = 'tonline_spider'
    allowed_domains = ['t-online.de']
    start_urls = ['https://www.t-online.de/']

    # Initialize the Queue and the Set
    visited_links = set()  # Set to keep track of processed links
    link_queue = Queue()   # FIFO Queue to store links

    # Set the date threshold for filtering articles
    date_threshold = datetime(2017, 1, 1, tzinfo=timezone.utc)

    def start_requests(self):
        """Initial request handler that starts the crawling process.

        This method iterates over the `start_urls`, adds them to the `link_queue`,
        and begins processing links.

        Yields:
            scrapy.http.Request: A request object for each starting URL, processed by
            the `process_links` method.
        """
        for url in self.start_urls:
            self.link_queue.put(url)

        # Start the link processing function
        yield from self.process_links()

    def process_links(self):
        """Processes links in the queue and sends requests for each.

        This method retrieves links from the queue, checks if they have been
        visited before, and then sends a request to `parse_article` for each
        unvisited link.

        Yields:
            scrapy.http.Request: A request object for each unvisited link in the queue.
        """
        while not self.link_queue.empty():
            next_link = self.link_queue.get()

            if next_link not in self.visited_links:
                self.visited_links.add(next_link)

            # Create a request and pass it to the `parse_article` method
            yield Request(url=next_link, callback=self.parse_article)

    def parse_frontpage(self, response):
        """Parses the front page of the ZDF news site and extracts article links.

        This method scrapes all relevant article links from the front page, adds them
        to the `link_queue` for further processing, and continues processing the links.

        Args:
            response (scrapy.http.Response): The HTTP response object containing the front page content.

        Yields:
            scrapy.http.Request: A request object for each article link found on the front page.
        """
        links = response.xpath('//article//a/@href').getall()
        for link in links:
            absolute_url = response.urljoin(link)
            if absolute_url not in self.visited_links:
                self.link_queue.put(absolute_url)

        # Continue processing links in the queue
        yield from self.process_links()

    def parse_article(self, response):
        """Parses an article page and extracts its content.

        This method scrapes the article's text, extracts relevant links from the
        article page, and checks if the article meets the date threshold. If the
        article is valid, its content is returned in the form of a dictionary.

        Args:
            response (scrapy.http.Response): The HTTP response object containing the article's content.

        Yields:
            dict: A dictionary containing the article's URL, content, and published date if the
            article is valid (published after the date threshold).
        """
        article_url = response.url

        # Extract new links from the article page and add them to the queue
        links = response.xpath('//a/@href').getall()
        for link in links:
            absolute_url = response.urljoin(link)
            if absolute_url not in self.visited_links:
                self.link_queue.put(absolute_url)

        # Only process valid article URLs
        if '/podcasts/' not in article_url:
            # Extract the article's date
            article_date_str = response.css('span.text-manatee.text-12.leading-15::text').re_first(r'(\d{2}\.\d{2}\.\d{4})')  

            if article_date_str:
                # Convert the date string to a datetime object
                article_date = datetime.strptime(article_date_str, "%d.%m.%Y").replace(tzinfo=timezone.utc)
                # Check if the article's date is after the threshold
                if article_date >= self.date_threshold:
                    # Extract article content
                    article_text = response.xpath('//p[contains(@class, "text-18")]/text() | //p[contains(@class, "text-18")]//a/text()').getall()
                    article_text = ' '.join(article_text).strip()

                    # Return the article data if content exists
                    if article_text:
                        yield {
                            'url': article_url,
                            'content': article_text,
                            'date': article_date_str,
                        }

        # Continue processing links in the queue
        yield from self.process_links()
