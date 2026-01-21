<script lang="ts">
  import '../app.css';
</script>

<div class="layout">
  <header>
    <nav>
      <a href="/" class="logo">Bot Arena</a>
      <div class="nav-links">
        <a href="/">Dashboard</a>
        <a href="/games">Games</a>
        <a href="/openings">Openings</a>
        <a href="/stats">Stats</a>
        <a href="/match/new">New Match</a>
      </div>
    </nav>
  </header>

  <main>
    <slot />
  </main>
</div>

<style>
  .layout {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  header {
    background: var(--bg-secondary);
    padding: 1rem 2rem;
    border-bottom: 1px solid var(--accent);
  }

  nav {
    display: flex;
    align-items: center;
    gap: 2rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .logo {
    font-size: 1.5rem;
    font-weight: bold;
    color: var(--highlight);
  }

  .nav-links {
    display: flex;
    gap: 1.5rem;
  }

  .nav-links a {
    color: var(--text);
  }

  main {
    flex: 1;
    padding: 2rem;
    max-width: 1200px;
    margin: 0 auto;
    width: 100%;
  }
</style>
