document.addEventListener('DOMContentLoaded', () => {
    // Navbar scroll effect
    const nav = document.querySelector('nav');
    window.addEventListener('scroll', () => {
        if (window.scrollY > 50) {
            nav.classList.add('scrolled');
        } else {
            nav.classList.remove('scrolled');
        }
    });

    // Terminal Typing Effect
    const terminalBody = document.getElementById('terminal-body');
    const commands = [
        { text: 'portforge', delay: 1000 },
        { text: 'portforge --all', delay: 3000 },
        { text: 'portforge inspect 3000', delay: 2000 }
    ];

    const terminalMocks = {
        'portforge': `
<span class="output">[SCANNING] Ports...</span>
<span class="output">PORT   PID     PROCESS        PROJECT     STATUS</span>
<span class="output">3000   12450   node           Next.js     ● Healthy</span>
<span class="output">8080   32101   python         FastAPI     ● Healthy</span>
<span class="output">9000   54122   portforge      Rust        ? Unknown</span>`,
        'portforge --all': `
<span class="output">[SCANNING] All listening ports...</span>
<span class="output">PORT   PID     PROCESS        PROJECT     STATUS</span>
<span class="output">22     800     sshd           System      -</span>
<span class="output">3000   12450   node           Next.js     ● Healthy</span>
<span class="output">5432   1540    postgres       DB          ● Healthy</span>
<span class="output">8080   32101   python         FastAPI     ● Healthy</span>`,
        'portforge inspect 3000': `
<span class="output">[INSPECT] Port 3000</span>
<span class="output">Process: node (PID 12450)</span>
<span class="output">Project: portforge-ui (Next.js)</span>
<span class="output">Framework Detect: package.json (next: v14.0.1)</span>
<span class="output">Health: ● Healthy (200 OK)</span>
<span class="output">Uptime: 2h 14m</span>`
    };

    async fn typeCommand(text, container) {
        return new Promise(resolve => {
            let i = 0;
            const prompt = document.createElement('div');
            prompt.innerHTML = '<span class="prompt">~$</span> <span class="command"></span><span class="cursor"></span>';
            container.appendChild(prompt);
            const cmdSpan = prompt.querySelector('.command');
            
            const interval = setInterval(() => {
                cmdSpan.textContent += text[i];
                i++;
                if (i === text.length) {
                    clearInterval(interval);
                    prompt.querySelector('.cursor').remove();
                    resolve();
                }
            }, 50);
        });
    }

    async fn runTerminal() {
        terminalBody.innerHTML = '';
        for (const cmd of commands) {
            await typeCommand(cmd.text, terminalBody);
            const output = document.createElement('div');
            output.innerHTML = terminalMocks[cmd.text];
            terminalBody.appendChild(output);
            await new Promise(r => setTimeout(r, cmd.delay));
        }
        setTimeout(runTerminal, 5000);
    }

    if (terminalBody) runTerminal();

    // Copy to Clipboard
    window.copyCode = (id) => {
        const code = document.getElementById(id).textContent;
        navigator.clipboard.writeText(code).then(() => {
            const btn = document.querySelector(`[onclick="copyCode('${id}')"]`);
            const originalText = btn.innerHTML;
            btn.innerHTML = '<span style="color: var(--primary)">✓</span>';
            setTimeout(() => { btn.innerHTML = originalText; }, 2000);
        });
    }

    // Tabs
    window.switchTab = (method) => {
        document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
        document.querySelectorAll('.install-content').forEach(c => c.style.display = 'none');
        
        document.querySelector(`.tab[onclick*="${method}"]`).classList.add('active');
        document.getElementById(`install-${method}`).style.display = 'block';
    }

    // Reveal on Scroll
    const observerOptions = {
        threshold: 0.1
    };

    const observer = new IntersectionObserver((entries) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                entry.target.style.opacity = '1';
                entry.target.style.transform = 'translateY(0)';
            }
        });
    }, observerOptions);

    document.querySelectorAll('.glass-card').forEach(card => {
        card.style.opacity = '0';
        card.style.transform = 'translateY(30px)';
        card.style.transition = 'all 0.6s ease-out';
        observer.observe(card);
    });
});
