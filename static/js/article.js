let voted = false;
const upvoteBtn = document.querySelector('.vote-up');
const downvoteBtn = document.querySelector('.vote-down');
const voteScore = document.getElementById('vote-score');
let score = parseInt(voteScore.textContent);
const id = window.location.pathname.split('/').at(-1);

function getVotedArticles() {
    try {
        const data = document.cookie.split('; ').find(row => row.startsWith('voted_articles='));
        if (data) {
            const json = decodeURIComponent(data.split('=')[1]);
            return JSON.parse(json);
        }
    } catch (e) {
        console.error('Cookie error:', e);
    }
    return {};
}

function saveVotedArticle(articleId, option) {
    try {
        let voted = getVotedArticles();
        voted[articleId] = option;
        document.cookie = `voted_articles=${encodeURIComponent(JSON.stringify(voted))}; path=/; max-age=31536000`;
    } catch (e) {
        console.error('Not saved:', e);
    }
}

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
    const votedArticles = getVotedArticles();
    if (votedArticles[id]) return;
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
        saveVotedArticle(id, option);
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