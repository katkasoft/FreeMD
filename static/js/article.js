let voted = false;
const upvoteBtn = document.querySelector('.vote-up');
const downvoteBtn = document.querySelector('.vote-down');
const voteScore = document.getElementById('vote-score');
let score = parseInt(voteScore.textContent);
const id = window.location.pathname.split('/').at(-1);

const content = document.getElementById('content');
document.addEventListener("DOMContentLoaded", () => {
    content.innerHTML = marked.parse(content.innerHTML);
    document.getElementById('edit-btn').addEventListener('click', () => {
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
    const url = `/api/vote?option=${option}&id=${id}`;
    const formData = new URLSearchParams();
    formData.append('option', option);
    formData.append('id', id);
    fetch(url, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/x-www-form-urlencoded'
        },
        body: formData
    })
    .then(response => {
        if (!response.ok) {
            throw new Error(response.status);
        }
        return response.json();
    })
    .then(data => {
        if (data.score !== undefined) {
            voteScore.textContent = data.score;
            score = data.score;
        }
    })
    .catch(error => {
        console.error('Error: ', error);
        voted = false;
        if (option === 'up') {
            score -= 1;
            upvoteBtn.classList.remove('active');
        } else if (option === 'down') {
            score += 1;
            downvoteBtn.classList.remove('active');
        }
        voteScore.textContent = score;
    })
}