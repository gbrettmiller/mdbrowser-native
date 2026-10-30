@remote
Feature: Markdown fallbacks
  As a reader of the web of Markdown
  I want a page that isn't Markdown to show its Markdown version when the site has one
  So that I read the writing, not the page around it

  # Fixture: the routes of remote.feature, plus
  #   /article       text/html      /article.md   text/markdown "# Article In Markdown"
  #   /linked        text/html with Link: </alt/linked.md>; rel="alternate"; type="text/markdown"
  #   /alt/linked.md text/markdown  "# Linked Alternate"   /linked.md text/markdown "# Sibling Loses"
  #   /bare          text/html, no Markdown anywhere       /llms.txt    text/markdown "# Site Index"

  Scenario: The .md sibling is shown, and the reader is told
    When I enter the URL of "/article"
    Then I see the document titled "Article In Markdown"
    And the address shows the URL of "/article.md"
    And History lists the URL of "/article.md" titled "Article In Markdown"
    And the page says it was found instead of the URL of "/article"

  Scenario: A Link alternate wins over the sibling
    When I enter the URL of "/linked"
    Then I see the document titled "Linked Alternate"
    And the address shows the URL of "/alt/linked.md"

  Scenario: The system browser is offered for the original
    Given I am viewing the URL of "/article"
    When I choose to open it in the browser
    Then the system opened the URL of "/article"

  Scenario: Plain Markdown shows no banner
    When I enter the URL of "/index.md"
    Then the page does not say it was found instead

  Scenario: Going Back to a fallback page shows no banner
    Given I viewed the URL of "/index.md" then the URL of "/article"
    When I go back
    And I go forward
    Then I see the document titled "Article In Markdown"
    And the page does not say it was found instead

  Scenario: With nothing to fall back to, an llms.txt is offered, not substituted
    Given the site has an llms.txt
    When I enter the URL of "/bare"
    Then I see the error "notMarkdownWithLlms"
    When I choose the site's llms.txt
    Then I see the document titled "Site Index"
    And the address shows the URL of "/llms.txt"

  Scenario: With no llms.txt, none is offered
    When I enter the URL of "/bare"
    Then I see the error "notMarkdown"
    And I am not offered the site's llms.txt

  Scenario: llms.txt is only looked for when nothing else was found
    Given the site has an llms.txt
    When I enter the URL of "/article"
    Then the site was not asked for "/llms.txt"

  Scenario: Other failures never fall back
    When I enter the URL of "/missing.md"
    Then I see the error "notFound"
    And I am not offered the site's llms.txt
