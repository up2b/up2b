import React, { useEffect, useState } from "react";
import { message, Flex, Spin, FloatButton, Empty } from "antd";
import { SyncOutlined } from "@ant-design/icons";
import {
  setStorage,
  getConfig,
  getAllImages,
  getImagesInStorage,
  getUsingImageBed,
} from "~/lib";
import "./index.scss";
import { LazyImageCard } from "~/lazy";
import { suspense } from "~/advance";

const ImageList = () => {
  const [messageApi, contextHolder] = message.useMessage();

  const [imageBedCode, setImageBedCode] = useState<ManagerCode | null>(null);

  const [images, setImages] = useState<ImageResponseItem[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (imageBedCode) return;

    getUsingImageBed()
      .then((c) => setImageBedCode(c))
      .finally(() => setLoading(false));
  }, []);

  useEffect(() => {
    if (!imageBedCode) return;

    const cached = getImagesInStorage(imageBedCode);

    if (!cached?.length) updateImageList();
    else setImages(cached);
  }, [imageBedCode]);

  useEffect(() => {
    if (!imageBedCode || images.length === 0) return;

    setStorage(imageBedCode, images);
  }, [images, imageBedCode]);

  const updateImageList = async () => {
    const config = await getConfig();
    if (!config) {
      messageApi.error("配置为空");
      return;
    }

    setLoading(true);

    try {
      const list = await getAllImages();

      list.reverse();

      setLoading(false);

      setStorage(imageBedCode!, list);

      setImages(list);
    } catch (e) {
      const error = String(e);
      if (error === "资源不存在") {
        // 只有 github 有这个错误信息
        setLoading(false);
        message.warning("目录为空，请先上传一张图片");
      }
    }
  };

  const afterDeleting = (url: string) => {
    setImages((pre) => {
      const newImages = pre.filter((v) => v.url !== url);

      setStorage(imageBedCode!, newImages);

      return newImages;
    });
  };

  return (
    <Spin spinning={loading} className="loading-list">
      <div
        id="image-list"
        className={images.length ? undefined : "image-list__empty"}
      >
        {contextHolder}

        {!images.length ? (
          <Empty description={false} />
        ) : (
          <Flex wrap="wrap" gap="small" justify="center">
            {images.map((item, index) => (
              <div key={index} className="image-card-container">
                {suspense(
                  <LazyImageCard
                    url={item.url}
                    thumb={item.thumb}
                    status={{
                      type: "success",
                      deleteId: item.deleted_id,
                      afterDeleting: afterDeleting,
                    }}
                    messageApi={messageApi}
                  />,
                )}
              </div>
            ))}
          </Flex>
        )}

        <FloatButton
          icon={<SyncOutlined />}
          tooltip="刷新列表"
          onClick={updateImageList}
        />
      </div>
    </Spin>
  );
};

export default ImageList;
