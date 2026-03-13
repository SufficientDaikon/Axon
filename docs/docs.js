(function () {
  "use strict";

  // ── Utility ──────────────────────────────────────────────────────────
  function debounce(fn, ms) {
    var timer;
    return function () {
      var ctx = this, args = arguments;
      clearTimeout(timer);
      timer = setTimeout(function () { fn.apply(ctx, args); }, ms);
    };
  }

  function $(sel, root) { return (root || document).querySelector(sel); }
  function $$(sel, root) { return Array.from((root || document).querySelectorAll(sel)); }

  // ── References ───────────────────────────────────────────────────────
  var sidebar      = $("#sidebar");
  var sidebarToggle = $("#sidebar-toggle");
  var searchInput  = $("#search-input");
  var tocContainer = $("#toc");
  var allPages     = $$("article.doc-page");
  var allNavLinks  = $$("a.nav-link");
  var allGroups    = $$(".nav-group");

  var DEFAULT_PAGE = "guide/getting-started";
  var currentPageId = null;
  var tocObserver   = null;

  // ── 1. Hash-based Routing ────────────────────────────────────────────
  function parseHash() {
    var raw = location.hash.replace(/^#\/?/, "");
    if (!raw) return { page: DEFAULT_PAGE, section: null };
    // Support #page-id/section-id  where page-id may itself contain "/"
    // Strategy: find the longest page-id prefix that matches a doc-page
    var parts = raw.split("/");
    for (var i = parts.length; i >= 1; i--) {
      var candidatePage = parts.slice(0, i).join("/");
      var article = $('article.doc-page[data-page="' + candidatePage + '"]');
      if (article) {
        var section = parts.slice(i).join("/") || null;
        return { page: candidatePage, section: section };
      }
    }
    // No match – try exact full string as page, no section
    return { page: raw, section: null };
  }

  function showPage(pageId, sectionId, pushState) {
    var target = $('article.doc-page[data-page="' + pageId + '"]');
    if (!target) {
      // Fallback to default
      pageId = DEFAULT_PAGE;
      sectionId = null;
      target = $('article.doc-page[data-page="' + pageId + '"]');
    }
    if (!target) return;

    // If same page, just scroll to section
    if (currentPageId === pageId) {
      if (sectionId) scrollToSection(target, sectionId);
      return;
    }

    var previous = $("article.doc-page.active");

    // ── 7. Page Transitions ──────────────────────────────────────────
    if (previous && previous !== target) {
      previous.classList.add("fade-out");
      previous.classList.remove("fade-in");
      // After transition ends, swap visibility
      setTimeout(function () {
        previous.classList.remove("active", "fade-out");
        activateTarget(target, pageId, sectionId);
      }, 150);
    } else {
      if (previous) previous.classList.remove("active", "fade-in", "fade-out");
      activateTarget(target, pageId, sectionId);
    }
  }

  function activateTarget(target, pageId, sectionId) {
    target.classList.add("active", "fade-in");
    setTimeout(function () { target.classList.remove("fade-in"); }, 150);

    currentPageId = pageId;
    updateSidebarActive(pageId);
    buildTOC(target);

    if (sectionId) {
      // Small delay so the page is rendered before scrolling
      requestAnimationFrame(function () { scrollToSection(target, sectionId); });
    } else {
      window.scrollTo({ top: 0 });
    }
  }

  function scrollToSection(article, sectionId) {
    // Look for heading with matching id inside the article
    var heading = article.querySelector("#" + CSS.escape(sectionId));
    if (!heading) {
      // Try case-insensitive slug match
      var headings = article.querySelectorAll("h2, h3, h4");
      for (var i = 0; i < headings.length; i++) {
        if (slugify(headings[i].textContent) === sectionId) {
          heading = headings[i];
          break;
        }
      }
    }
    if (heading) {
      heading.scrollIntoView({ behavior: "smooth", block: "start" });
    }
  }

  function slugify(text) {
    return text.toLowerCase().trim()
      .replace(/[^\w\s-]/g, "")
      .replace(/[\s_]+/g, "-")
      .replace(/^-+|-+$/g, "");
  }

  function onHashChange() {
    var parsed = parseHash();
    showPage(parsed.page, parsed.section);
  }

  window.addEventListener("hashchange", onHashChange);

  // ── 2. Sidebar Navigation ───────────────────────────────────────────
  function updateSidebarActive(pageId) {
    allNavLinks.forEach(function (link) {
      link.classList.toggle("active", link.getAttribute("data-page") === pageId);
    });
    // Auto-expand the group containing the active link
    allGroups.forEach(function (group) {
      var hasActive = group.querySelector('a.nav-link[data-page="' + pageId + '"]');
      if (hasActive) {
        group.classList.remove("collapsed");
      }
    });
  }

  // Group header collapse/expand
  allGroups.forEach(function (group) {
    var header = group.querySelector(".nav-group-header");
    if (!header) return;
    header.addEventListener("click", function (e) {
      e.preventDefault();
      group.classList.toggle("collapsed");
    });
    // Keyboard accessibility
    header.setAttribute("role", "button");
    header.setAttribute("tabindex", "0");
    header.addEventListener("keydown", function (e) {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        group.classList.toggle("collapsed");
      }
    });
  });

  // Nav link clicks – just set the hash; hashchange handler does the rest
  allNavLinks.forEach(function (link) {
    link.addEventListener("click", function (e) {
      e.preventDefault();
      var page = link.getAttribute("data-page") || link.getAttribute("href").replace(/^#/, "");
      location.hash = "#" + page;

      // ── 4. Close sidebar on mobile ──────────────────────────────
      if (window.innerWidth < 768) {
        sidebar.classList.remove("open");
      }
    });
  });

  // ── 3. Search ────────────────────────────────────────────────────────
  var performSearch = debounce(function () {
    var query = searchInput.value.trim().toLowerCase();

    if (!query) {
      // Show everything, restore collapse states
      allNavLinks.forEach(function (link) { link.style.display = ""; });
      allGroups.forEach(function (group) {
        group.style.display = "";
        // Keep the group with the active link expanded
        var hasActive = group.querySelector("a.nav-link.active");
        if (!hasActive) {
          // Restore to whatever collapsed state it was in; don't force
        }
      });
      return;
    }

    allGroups.forEach(function (group) {
      var anyVisible = false;
      var links = group.querySelectorAll("a.nav-link");
      links.forEach(function (link) {
        var text = link.textContent.toLowerCase();
        var match = text.indexOf(query) !== -1;
        link.style.display = match ? "" : "none";
        if (match) anyVisible = true;
      });
      // Hide entire group if no matching links
      group.style.display = anyVisible ? "" : "none";
      // Auto-expand groups that have matches
      if (anyVisible) {
        group.classList.remove("collapsed");
      }
    });
  }, 150);

  if (searchInput) {
    searchInput.addEventListener("input", performSearch);
  }

  // ── 4. Mobile Sidebar Toggle ─────────────────────────────────────────
  if (sidebarToggle && sidebar) {
    sidebarToggle.addEventListener("click", function () {
      sidebar.classList.toggle("open");
    });
    // Close sidebar when clicking outside on mobile
    document.addEventListener("click", function (e) {
      if (window.innerWidth < 768 &&
          sidebar.classList.contains("open") &&
          !sidebar.contains(e.target) &&
          e.target !== sidebarToggle &&
          !sidebarToggle.contains(e.target)) {
        sidebar.classList.remove("open");
      }
    });
  }

  // ── 5. Code Copy Buttons ─────────────────────────────────────────────
  function initCopyButtons() {
    $$("pre").forEach(function (pre) {
      // Don't add duplicates
      if (pre.querySelector(".copy-btn")) return;

      var btn = document.createElement("button");
      btn.className = "copy-btn";
      btn.textContent = "Copy";
      btn.setAttribute("aria-label", "Copy code to clipboard");

      // Make the pre position:relative if not already
      var pos = getComputedStyle(pre).position;
      if (pos === "static") pre.style.position = "relative";

      btn.addEventListener("click", function () {
        var code = pre.querySelector("code");
        var text = code ? code.textContent : pre.textContent;

        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(text).then(function () {
            showCopied(btn);
          }, function () {
            fallbackCopy(text, btn);
          });
        } else {
          fallbackCopy(text, btn);
        }
      });

      pre.appendChild(btn);
    });
  }

  function fallbackCopy(text, btn) {
    var ta = document.createElement("textarea");
    ta.value = text;
    ta.style.position = "fixed";
    ta.style.opacity = "0";
    document.body.appendChild(ta);
    ta.select();
    try {
      document.execCommand("copy");
      showCopied(btn);
    } catch (e) { /* silently fail */ }
    document.body.removeChild(ta);
  }

  function showCopied(btn) {
    btn.textContent = "Copied!";
    btn.classList.add("copied");
    setTimeout(function () {
      btn.textContent = "Copy";
      btn.classList.remove("copied");
    }, 2000);
  }

  // ── 6. Table of Contents Generation ──────────────────────────────────
  function buildTOC(article) {
    if (!tocContainer) return;

    // Clean up previous observer
    if (tocObserver) {
      tocObserver.disconnect();
      tocObserver = null;
    }

    tocContainer.innerHTML = "";

    var headings = article.querySelectorAll("h2, h3");
    if (headings.length === 0) return;

    var tocTitle = document.createElement("div");
    tocTitle.className = "toc-title";
    tocTitle.textContent = "On this page";
    tocContainer.appendChild(tocTitle);

    var rootList = document.createElement("ul");
    rootList.className = "toc-list";
    tocContainer.appendChild(rootList);

    var currentH2Item = null;
    var currentSubList = null;
    var tocLinks = [];

    headings.forEach(function (heading) {
      // Ensure heading has an id
      if (!heading.id) {
        heading.id = slugify(heading.textContent);
      }

      var li = document.createElement("li");
      li.className = "toc-item";
      var a = document.createElement("a");
      a.className = "toc-link";
      a.href = "#" + currentPageId + "/" + heading.id;
      a.textContent = heading.textContent;
      a.setAttribute("data-heading-id", heading.id);

      a.addEventListener("click", function (e) {
        e.preventDefault();
        heading.scrollIntoView({ behavior: "smooth", block: "start" });
        // Update hash without triggering page reload
        history.replaceState(null, "", "#" + currentPageId + "/" + heading.id);
      });

      li.appendChild(a);
      tocLinks.push({ link: a, heading: heading });

      if (heading.tagName === "H2") {
        rootList.appendChild(li);
        currentH2Item = li;
        currentSubList = null;
      } else if (heading.tagName === "H3") {
        if (!currentSubList) {
          currentSubList = document.createElement("ul");
          currentSubList.className = "toc-sublist";
          if (currentH2Item) {
            currentH2Item.appendChild(currentSubList);
          } else {
            rootList.appendChild(li);
            return;
          }
        }
        currentSubList.appendChild(li);
      }
    });

    // IntersectionObserver for active TOC item
    if (tocLinks.length > 0 && typeof IntersectionObserver !== "undefined") {
      var headingElements = tocLinks.map(function (item) { return item.heading; });

      tocObserver = new IntersectionObserver(function (entries) {
        // Find the topmost visible heading
        var visibleEntries = entries.filter(function (entry) {
          return entry.isIntersecting;
        });

        if (visibleEntries.length > 0) {
          // Sort by position in viewport (top first)
          visibleEntries.sort(function (a, b) {
            return a.boundingClientRect.top - b.boundingClientRect.top;
          });

          var activeId = visibleEntries[0].target.id;
          tocLinks.forEach(function (item) {
            item.link.classList.toggle("active", item.heading.id === activeId);
          });
        }
      }, {
        root: null,
        rootMargin: "-64px 0px -60% 0px",
        threshold: 0
      });

      headingElements.forEach(function (h) {
        tocObserver.observe(h);
      });

      // Also handle scroll for cases where no heading is intersecting
      // (e.g., between two headings that are far apart)
      var scrollHandler = debounce(function () {
        var scrollTop = window.scrollY || document.documentElement.scrollTop;
        var offset = 100; // Account for fixed header
        var current = null;

        for (var i = headingElements.length - 1; i >= 0; i--) {
          var rect = headingElements[i].getBoundingClientRect();
          if (rect.top <= offset) {
            current = headingElements[i];
            break;
          }
        }

        if (current) {
          tocLinks.forEach(function (item) {
            item.link.classList.toggle("active", item.heading.id === current.id);
          });
        }
      }, 50);

      window.addEventListener("scroll", scrollHandler, { passive: true });

      // Store the handler so we can remove it when rebuilding
      tocContainer._scrollHandler = scrollHandler;
    }
  }

  // Clean up scroll handler before rebuilding TOC
  var origBuildTOC = buildTOC;
  buildTOC = function (article) {
    if (tocContainer && tocContainer._scrollHandler) {
      window.removeEventListener("scroll", tocContainer._scrollHandler);
      tocContainer._scrollHandler = null;
    }
    origBuildTOC(article);
  };

  // ── Initialize ───────────────────────────────────────────────────────
  document.addEventListener("DOMContentLoaded", function () {
    // Re-query in case DOM wasn't ready during initial assignment
    if (!sidebar) sidebar = $("#sidebar");
    if (!sidebarToggle) sidebarToggle = $("#sidebar-toggle");
    if (!searchInput) searchInput = $("#search-input");
    if (!tocContainer) tocContainer = $("#toc");

    if (searchInput) searchInput.addEventListener("input", performSearch);

    initCopyButtons();
    onHashChange();
  });

  // If DOMContentLoaded already fired (script at bottom of body)
  if (document.readyState !== "loading") {
    initCopyButtons();
    onHashChange();
  }

  // ── Section 7: Interactive Code Tooltips ─────────────────────────
  var TIPS = {
    "fn":       ["Keyword", "Declares a function — a reusable block of code you can call by name.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Functions", "MDN: Functions"],
    "let":      ["Keyword", "Creates a variable — a named container that stores a value.", "https://developer.mozilla.org/en-US/docs/Learn/JavaScript/First_steps/Variables", "MDN: Variables"],
    "mut":      ["Keyword", "Makes a variable mutable — means you can change its value later.", "https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html", "Rust Book: Mutability"],
    "return":   ["Keyword", "Sends a value back from a function to whoever called it.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/return", "MDN: Return"],
    "if":       ["Keyword", "Runs code only when a condition is true.", "https://developer.mozilla.org/en-US/docs/Learn/JavaScript/Building_blocks/conditionals", "MDN: Conditionals"],
    "else":     ["Keyword", "Runs this code when the if condition was false.", "https://developer.mozilla.org/en-US/docs/Learn/JavaScript/Building_blocks/conditionals", "MDN: Conditionals"],
    "for":      ["Keyword", "A loop — repeats code for each item in a collection.", "https://developer.mozilla.org/en-US/docs/Learn/JavaScript/Building_blocks/Looping_code", "MDN: Loops"],
    "in":       ["Keyword", "Specifies what collection to loop over.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/for...in", "MDN: for...in"],
    "while":    ["Keyword", "A loop — keeps repeating as long as a condition stays true.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/while", "MDN: While loops"],
    "match":    ["Keyword", "Checks a value against multiple patterns and runs the matching one — like a powerful switch.", "https://doc.rust-lang.org/book/ch06-02-match.html", "Rust Book: Match"],
    "struct":   ["Keyword", "Defines a custom data type that groups related values together.", "https://doc.rust-lang.org/book/ch05-01-defining-structs.html", "Rust Book: Structs"],
    "enum":     ["Keyword", "Defines a type that can be one of several different variants.", "https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html", "Rust Book: Enums"],
    "impl":     ["Keyword", "Adds methods (functions) to a struct or enum.", "https://doc.rust-lang.org/book/ch05-03-method-syntax.html", "Rust Book: Methods"],
    "trait":    ["Keyword", "Defines a set of behaviors that types can implement — like an interface or contract.", "https://doc.rust-lang.org/book/ch10-02-traits.html", "Rust Book: Traits"],
    "pub":      ["Keyword", "Makes something public — accessible from outside its module.", "https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html", "Rust Book: Modules"],
    "use":      ["Keyword", "Imports items from another module so you can use them here.", "https://doc.rust-lang.org/book/ch07-04-bringing-paths-into-scope-with-the-use-keyword.html", "Rust Book: Use"],
    "import":   ["Keyword", "Brings code from another file or package into the current file.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/import", "MDN: Import"],
    "true":     ["Keyword", "A boolean value — represents yes or on.", "https://developer.mozilla.org/en-US/docs/Glossary/Boolean", "MDN: Booleans"],
    "false":    ["Keyword", "A boolean value — represents no or off.", "https://developer.mozilla.org/en-US/docs/Glossary/Boolean", "MDN: Booleans"],
    "type":     ["Keyword", "Creates a name alias for another type — like a nickname.", "https://doc.rust-lang.org/book/ch19-04-advanced-types.html", "Rust Book: Type Aliases"],
    "self":     ["Keyword", "Refers to the current object or instance inside its own methods.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/this", "MDN: this/self"],
    "const":    ["Keyword", "Declares a constant — a value that can never be changed.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/const", "MDN: Const"],
    "static":   ["Keyword", "A value shared by all instances of a type, not tied to one object.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Classes/static", "MDN: Static"],
    "async":    ["Keyword", "Marks a function that can pause and wait for slow operations to finish.", "https://developer.mozilla.org/en-US/docs/Learn/JavaScript/Asynchronous/Promises", "MDN: Async"],
    "await":    ["Keyword", "Pauses execution until an async operation completes and gives a result.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/await", "MDN: Await"],
    "break":    ["Keyword", "Immediately exits the current loop.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/break", "MDN: Break"],
    "continue": ["Keyword", "Skips the rest of this loop iteration and jumps to the next one.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Statements/continue", "MDN: Continue"],
    "loop":     ["Keyword", "Repeats code forever until you explicitly stop it with break.", "https://doc.rust-lang.org/book/ch03-05-control-flow.html#repetition-with-loops", "Rust Book: Loops"],
    "move":     ["Keyword", "Transfers ownership of a value — the original can no longer use it.", "https://doc.rust-lang.org/book/ch04-01-what-is-ownership.html", "Rust Book: Ownership"],
    "where":    ["Keyword", "Adds constraints on generic types — specifies what capabilities they must have.", "https://doc.rust-lang.org/book/ch10-02-traits.html#clearer-trait-bounds-with-where-clauses", "Rust Book: Where"],
    "ref":      ["Keyword", "Creates a reference to a value instead of moving it.", "https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html", "Rust Book: References"],
    "as":       ["Keyword", "Converts a value from one type to another (type casting).", "https://doc.rust-lang.org/book/appendix-02-operators.html", "Rust Book: Operators"],
    "defer":    ["Keyword", "Schedules code to run when the current scope ends — great for cleanup.", "https://en.wikipedia.org/wiki/Resource_management_(computing)", "Wikipedia: Resource Management"],
    "grad":     ["Keyword", "Enables automatic gradient computation — essential for training ML models.", "https://www.youtube.com/watch?v=aircAruvnKk", "3Blue1Brown: Neural Networks"],
    "@gpu":     ["Keyword", "Tells the compiler to run this on the GPU for massively parallel computation.", "https://en.wikipedia.org/wiki/General-purpose_computing_on_graphics_processing_units", "Wikipedia: GPGPU"],
    "@cpu":     ["Keyword", "Tells the compiler to run this on the CPU (main processor).", "https://en.wikipedia.org/wiki/Central_processing_unit", "Wikipedia: CPU"],
    "@device":  ["Keyword", "Specifies which hardware device to run computation on.", "https://pytorch.org/docs/stable/tensor_attributes.html#torch.device", "PyTorch: Devices"],
    "@jit":     ["Keyword", "Just-In-Time compilation — compiles code at runtime for better performance.", "https://en.wikipedia.org/wiki/Just-in-time_compilation", "Wikipedia: JIT"],
    "@inline":  ["Keyword", "Hints the compiler to insert this function's code directly at call sites.", "https://en.wikipedia.org/wiki/Inline_expansion", "Wikipedia: Inlining"],
    "@pure":    ["Keyword", "Marks a function as pure — same inputs always give same outputs, no side effects.", "https://en.wikipedia.org/wiki/Pure_function", "Wikipedia: Pure Functions"],
    "@test":    ["Keyword", "Marks this function as a test that verifies code works correctly.", "https://doc.rust-lang.org/book/ch11-01-writing-tests.html", "Rust Book: Tests"],
    "@differentiable": ["Keyword", "Marks a function as differentiable — the compiler can compute its gradient automatically.", "https://en.wikipedia.org/wiki/Automatic_differentiation", "Wikipedia: Autodiff"],
    "Tensor":   ["Type", "A multi-dimensional array of numbers — the core data structure in machine learning.", "https://pytorch.org/docs/stable/tensors.html", "PyTorch: Tensors"],
    "int":      ["Type", "A whole number with no decimals, like 1, 42, or -7.", "https://developer.mozilla.org/en-US/docs/Glossary/Number", "MDN: Numbers"],
    "i32":      ["Type", "A 32-bit integer — a whole number that can be roughly ±2 billion.", "https://developer.mozilla.org/en-US/docs/Glossary/Number", "MDN: Numbers"],
    "i64":      ["Type", "A 64-bit integer — a whole number that can be extremely large.", "https://developer.mozilla.org/en-US/docs/Glossary/Number", "MDN: Numbers"],
    "Int32":    ["Type", "A 32-bit integer — a whole number with no decimals.", "https://developer.mozilla.org/en-US/docs/Glossary/Number", "MDN: Numbers"],
    "Int64":    ["Type", "A 64-bit integer — a very large whole number.", "https://developer.mozilla.org/en-US/docs/Glossary/Number", "MDN: Numbers"],
    "float":    ["Type", "A decimal number like 3.14 or -0.5.", "https://developer.mozilla.org/en-US/docs/Glossary/Number", "MDN: Numbers"],
    "f32":      ["Type", "A 32-bit floating point — a decimal number with ~7 digits of precision.", "https://en.wikipedia.org/wiki/Single-precision_floating-point_format", "Wikipedia: Float32"],
    "f64":      ["Type", "A 64-bit floating point — a decimal number with ~15 digits of precision.", "https://en.wikipedia.org/wiki/Double-precision_floating-point_format", "Wikipedia: Float64"],
    "Float32":  ["Type", "A 32-bit decimal number — commonly used in ML for efficiency.", "https://en.wikipedia.org/wiki/Single-precision_floating-point_format", "Wikipedia: Float32"],
    "Float64":  ["Type", "A 64-bit decimal number — higher precision for scientific computing.", "https://en.wikipedia.org/wiki/Double-precision_floating-point_format", "Wikipedia: Float64"],
    "bool":     ["Type", "A true/false value — used for yes/no decisions in code.", "https://developer.mozilla.org/en-US/docs/Glossary/Boolean", "MDN: Booleans"],
    "string":   ["Type", "A piece of text, like a word or sentence, wrapped in quotes.", "https://developer.mozilla.org/en-US/docs/Learn/JavaScript/First_steps/Strings", "MDN: Strings"],
    "String":   ["Type", "A piece of text — a sequence of characters like words or sentences.", "https://developer.mozilla.org/en-US/docs/Learn/JavaScript/First_steps/Strings", "MDN: Strings"],
    "void":     ["Type", "Means nothing — the function does not give back any value.", "https://developer.mozilla.org/en-US/docs/Glossary/Void", "MDN: Void"],
    "Vec":      ["Type", "A growable list that can hold multiple values of the same type.", "https://doc.rust-lang.org/book/ch08-01-vectors.html", "Rust Book: Vectors"],
    "Array":    ["Type", "A fixed-size collection of values stored in order.", "https://developer.mozilla.org/en-US/docs/Learn/JavaScript/First_steps/Arrays", "MDN: Arrays"],
    "Option":   ["Type", "A value that might exist (Some) or might be empty (None) — prevents crashes from missing data.", "https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html#the-option-enum-and-its-advantages-over-null-values", "Rust Book: Option"],
    "Result":   ["Type", "Represents success (Ok) or failure (Err) — forces you to handle errors.", "https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html", "Rust Book: Result"],
    "Self":     ["Type", "Refers to the current type itself — used inside impl blocks.", "https://doc.rust-lang.org/book/ch05-03-method-syntax.html", "Rust Book: Methods"],
    "Shape":    ["Type", "Describes tensor dimensions, like [3, 4] for a 3-row by 4-column matrix.", "https://numpy.org/doc/stable/reference/generated/numpy.shape.html", "NumPy: Shapes"],
    "Device":   ["Type", "Specifies where computation runs — on the CPU or a GPU.", "https://pytorch.org/docs/stable/tensor_attributes.html#torch.device", "PyTorch: Devices"],
    "Map":      ["Type", "A collection of key-value pairs — look up values by their key.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Map", "MDN: Map"],
    "Set":      ["Type", "A collection of unique values — no duplicates allowed.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Set", "MDN: Set"],
    "Model":    ["Type", "A machine learning model — a trained system that makes predictions from data.", "https://en.wikipedia.org/wiki/Machine_learning#Models", "Wikipedia: ML Models"],
    "Layer":    ["Type", "A building block of neural networks — transforms input data step by step.", "https://en.wikipedia.org/wiki/Artificial_neural_network", "Wikipedia: Neural Networks"],
    "Optimizer":["Type", "An algorithm that adjusts model parameters during training to reduce error.", "https://pytorch.org/docs/stable/optim.html", "PyTorch: Optimizers"],
    "Loss":     ["Type", "A function that measures how wrong the model's predictions are.", "https://pytorch.org/docs/stable/nn.html#loss-functions", "PyTorch: Loss Functions"],
    "Gradient": ["Type", "The direction and rate of change — tells the model which way to adjust.", "https://www.youtube.com/watch?v=IHZwWFHWa-w", "3Blue1Brown: Gradient Descent"],
    "->":       ["Operator", "Shows what type of value a function gives back (returns).", "https://doc.rust-lang.org/book/ch03-03-how-functions-work.html#functions-with-return-values", "Rust Book: Return Types"],
    "=>":       ["Operator", "Points to the code that runs when a pattern matches.", "https://doc.rust-lang.org/book/ch06-02-match.html", "Rust Book: Match Arms"],
    "::":       ["Operator", "Accesses something inside a module or type — like opening a folder.", "https://doc.rust-lang.org/book/ch07-03-paths-for-referring-to-an-item-in-the-module-tree.html", "Rust Book: Paths"],
    "..":       ["Operator", "Creates a range of values, like 0..10 means 0 through 9.", "https://doc.rust-lang.org/book/ch03-05-control-flow.html#looping-through-a-collection-with-for", "Rust Book: Ranges"],
    "@":        ["Operator", "An annotation marker — adds special behavior to the code below it.", "https://en.wikipedia.org/wiki/Decorator_pattern", "Wikipedia: Decorators"],
    "&":        ["Operator", "Borrows a value — lets you use it without taking ownership.", "https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html", "Rust Book: Borrowing"],
    "&mut":     ["Operator", "Mutably borrows a value — lets you both read and modify it.", "https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html#mutable-references", "Rust Book: Mutable References"],
    "=":        ["Operator", "Assigns (stores) a value into a variable.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Assignment", "MDN: Assignment"],
    "==":       ["Operator", "Checks if two values are equal (comparison, not assignment).", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Equality", "MDN: Equality"],
    "!=":       ["Operator", "Checks if two values are NOT equal.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Inequality", "MDN: Inequality"],
    "+=":       ["Operator", "Adds to the current value and stores the result. x += 5 means x = x + 5.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Addition_assignment", "MDN: += Operator"],
    "*":        ["Operator", "Multiplies two values together.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Multiplication", "MDN: Multiplication"],
    "/":        ["Operator", "Divides one value by another.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Division", "MDN: Division"],
    ">":        ["Operator", "Checks if the left value is greater than the right.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Greater_than", "MDN: Comparison"],
    "<":        ["Operator", "Checks if the left value is less than the right.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Less_than", "MDN: Comparison"],
    "!":        ["Operator", "Negates a value — turns true to false and vice versa.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Operators/Logical_NOT", "MDN: NOT"],
    "?":        ["Operator", "Propagates errors — if this fails, return the error immediately.", "https://doc.rust-lang.org/book/ch09-02-recoverable-errors-with-result.html#a-shortcut-for-propagating-errors-the--operator", "Rust Book: ? Operator"]
  };

  var CLASS_FALLBACKS = {
    "kw":  ["Keyword",  "A keyword — a special reserved word with a built-in meaning in the language.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#keywords", "MDN: Keywords"],
    "typ": ["Type",     "A type — defines what kind of data a value can hold.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Data_structures", "MDN: Data Types"],
    "fn":  ["Function", "A function call — runs a named, reusable piece of code.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Functions", "MDN: Functions"],
    "str": ["String",   "A string literal — a piece of text written directly in quotes.", "https://developer.mozilla.org/en-US/docs/Learn/JavaScript/First_steps/Strings", "MDN: Strings"],
    "num": ["Number",   "A number literal — a numeric value written directly in code.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Number", "MDN: Numbers"],
    "cmt": ["Comment",  "A comment — a note for humans that the computer completely ignores.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types#comments", "MDN: Comments"],
    "op":  ["Operator", "An operator — a symbol that performs an action like math or comparison.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Expressions_and_operators", "MDN: Operators"],
    "p":   ["Punctuation", "Punctuation — structural characters like brackets, commas, and semicolons.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Grammar_and_types", "MDN: Syntax"],
    "pr":  ["Property", "A property — a named piece of data or method that belongs to an object.", "https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide/Working_with_objects", "MDN: Properties"]
  };

  var CAT_CSS = {
    "Keyword": "cat-keyword", "Type": "cat-type", "Function": "cat-function",
    "String": "cat-string", "Number": "cat-number", "Comment": "cat-comment",
    "Operator": "cat-operator", "Property": "cat-property", "Punctuation": "cat-operator"
  };

  // Create tooltip element
  var tip = document.createElement("div");
  tip.className = "code-tooltip";
  tip.innerHTML = '<div class="code-tooltip-category"></div><div class="code-tooltip-token"></div><div class="code-tooltip-text"></div><a class="code-tooltip-link" target="_blank" rel="noopener"></a>';
  document.body.appendChild(tip);

  var tipCat = tip.querySelector(".code-tooltip-category");
  var tipToken = tip.querySelector(".code-tooltip-token");
  var tipText = tip.querySelector(".code-tooltip-text");
  var tipLink = tip.querySelector(".code-tooltip-link");
  var hideTimer = null;
  var tipVisible = false;

  function getTipData(span) {
    var text = span.textContent.trim();
    var cls = span.className.split(" ")[0];
    var entry = TIPS[text];
    if (entry) return entry;
    return CLASS_FALLBACKS[cls] || null;
  }

  function showTip(span) {
    var data = getTipData(span);
    if (!data) return;
    clearTimeout(hideTimer);

    tipCat.textContent = data[0];
    tipCat.className = "code-tooltip-category " + (CAT_CSS[data[0]] || "cat-keyword");
    tipToken.textContent = span.textContent.trim();
    tipText.textContent = data[1];
    tipLink.textContent = data[3];
    tipLink.href = data[2];

    // Position
    var r = span.getBoundingClientRect();
    var tw = 320;
    var above = r.top > 180;
    tip.className = "code-tooltip " + (above ? "above" : "below");

    // Make visible to measure
    tip.style.visibility = "hidden";
    tip.style.opacity = "0";
    tip.style.pointerEvents = "none";
    tip.classList.add("visible");

    var left = r.left + r.width / 2 - tw / 2;
    if (left < 8) left = 8;
    if (left + tw > window.innerWidth - 8) left = window.innerWidth - tw - 8;

    tip.style.left = left + "px";
    tip.style.width = tw + "px";

    if (above) {
      tip.style.top = (r.top - tip.offsetHeight - 10) + "px";
    } else {
      tip.style.top = (r.bottom + 10) + "px";
    }

    tip.style.visibility = "";
    tip.style.opacity = "";
    tip.style.pointerEvents = "";
    tipVisible = true;
  }

  function hideTip() {
    hideTimer = setTimeout(function() {
      tip.classList.remove("visible");
      tipVisible = false;
    }, 200);
  }

  // Event delegation for hover
  document.addEventListener("mouseover", function(e) {
    var span = e.target;
    if (span.tagName !== "SPAN" || !span.className) return;
    var parent = span.closest("pre code");
    if (!parent) return;
    showTip(span);
  });

  document.addEventListener("mouseout", function(e) {
    var span = e.target;
    if (span.tagName !== "SPAN") return;
    var parent = span.closest("pre code");
    if (!parent) return;
    hideTip();
  });

  // Keep tooltip open when hovering the tooltip itself
  tip.addEventListener("mouseenter", function() {
    clearTimeout(hideTimer);
  });
  tip.addEventListener("mouseleave", function() {
    hideTip();
  });

  // Touch support: tap to show, tap outside to dismiss
  document.addEventListener("click", function(e) {
    var span = e.target;
    if (span.tagName === "SPAN" && span.className && span.closest("pre code")) {
      e.preventDefault();
      if (tipVisible && tipToken.textContent === span.textContent.trim()) {
        tip.classList.remove("visible");
        tipVisible = false;
      } else {
        showTip(span);
      }
      return;
    }
    if (!tip.contains(e.target) && tipVisible) {
      tip.classList.remove("visible");
      tipVisible = false;
    }
  });

  // Escape to dismiss
  document.addEventListener("keydown", function(e) {
    if (e.key === "Escape" && tipVisible) {
      tip.classList.remove("visible");
      tipVisible = false;
    }
  });

})();
