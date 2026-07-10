let voted = false;
const upvoteBtn = document.querySelector('.vote-up');
const downvoteBtn = document.querySelector('.vote-down');
const voteScore = document.getElementById('vote-score');
let score = parseInt(voteScore.textContent);

const content = document.getElementById('content');
document.addEventListener("DOMContentLoaded", () => {
    content.innerHTML = marked.parse(content.innerHTML);
    document.getElementById('edit-btn').addEventListener('click', () => {
        const id = window.location.pathname.split('/').at(-1);
        window.location.href = `/edit?id=${id}`;
    });
    upvoteBtn.addEventListener('click', () => vote('up'));
    downvoteBtn.addEventListener('click', () => vote('down'));
});

function vote(option) {
    if (voted) return;
    if (option === 'up') {
        upvoteBtn.classList.add('active');
        score += 1;
    } else if (option === 'down') {
        downvoteBtn.classList.add('active');
        score -= 1;
    }
    voted = true;
    voteScore.textContent = score;
}