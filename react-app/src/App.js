import React, {useEffect, useState} from 'react';
import {Card, Image} from 'antd';
import InfiniteScroll from 'react-infinite-scroll-component';

const InfiniteScrollImages = () => {
  const [images, setImages] = useState([]);
  const [hasMore, setHasMore] = useState(true);


  const params = new URLSearchParams(window.location.search);
  let queryUrl;
  if (window.location.href.indexOf("pixiv") !== -1) {
    queryUrl = new URL('api/pixiv/v1', window.location.origin);
  }else if (window.location.href.indexOf("local") !== -1) {
    queryUrl = new URL('api/pixiv/v2', window.location.origin);
  } else if (window.location.href.indexOf("photo") !== -1) {
    queryUrl = new URL('api/photo/v1', window.location.origin);
  } else if (window.location.href.indexOf("leg") !== -1) {
    queryUrl = new URL('api/leg/v1', window.location.origin);
  } else {
    queryUrl = new URL('api/anime/v1', window.location.origin);
  }
  //const queryUrl = new URL('api/anime/v1', window.location.origin);
  //const queryUrl = new URL('http://127.0.0.1:5800/pixiv');
  for (const [key, value] of params) {
    queryUrl.searchParams.append(key, value);
  }

  const fetchImages = () => {
    for (let i = 0; i < 5; i++) {

      console.log("开始处理*请求*");
      let request = fetch(queryUrl.toString())
        .then(res => res.blob());

      request.then(blob => {
        console.log("开始处理*blob");
        const imgUrl = URL.createObjectURL(blob);
        console.log("处理完*blob*\n");
        setHasMore(true);
        setImages(prevImages => [...prevImages, imgUrl]);
      })
        .catch(error => {
          console.error(error);
          setHasMore(false);
        });

    }
  };

  useEffect(() => {
    fetchImages();
  }, []);

  return (
    <InfiniteScroll
      dataLength={images.length}
      next={fetchImages}
      hasMore={hasMore}
      loader={<h4 style={{ textAlign: 'center' }}>Loading...</h4>}
    >
      {images.map((image, index) => (
        <Card key={index} style={{ width: '100%' }}>
          <h3 style={{ textAlign: 'center' }}>Image #{index + 1}</h3>
          <Image
            src={image}
            preview={true}
            alt={'Picture ' + (index + 1)}
            style={{
              width: '100%',
              height: 'auto',
              display: 'block',
              marginLeft: 'auto',
              marginRight: 'auto'
            }}
          />
        </Card>
      ))}
    </InfiniteScroll>
  );
};

export default InfiniteScrollImages;
