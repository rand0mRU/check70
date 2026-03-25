chrome.action.onClicked.addListener(async (tab) => {
  try {
    // Получаем HTML текущей страницы
    const [result] = await chrome.scripting.executeScript({
      target: { tabId: tab.id },
      func: () => {
        return {
          html: document.documentElement.outerHTML,
          url: window.location.href,
          title: document.title
        };
      }
    });
    
    // Отправляем на Python сервер
    const response = await fetch('http://localhost:60177/receive-html', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(result.result)
    });
    
    if (response.ok) {
      console.log('html sended');
      chrome.notifications.create({
        type: 'basic',
        iconUrl: 'icon.png',
        title: 'Success',
        message: 'HTML sended'
      });
    }
  } catch (error) {
    console.error('Ошибка:', error);
  }
});