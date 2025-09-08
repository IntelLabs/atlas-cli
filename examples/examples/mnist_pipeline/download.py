import argparse
import json
import logging
from pathlib import Path
from typing import Dict, Tuple

import torch.utils.data as data
from torchvision import transforms as tfs
from torchvision.datasets import MNIST


def download_mnist(path_to_data: Path, log: logging.Logger):
    log.info(f"Attempting to download MNIST to: {path_to_data}")
    _ = MNIST(path_to_data, train=True, download=True)
    _ = MNIST(path_to_data, train=False, download=True)


def load_mnist(
    path_to_data: Path, batch_size: int
) -> Tuple[data.DataLoader, data.DataLoader]:
    mean = [0.5]
    std = [0.5]
    transforms = tfs.Compose([tfs.ToTensor(), tfs.Normalize(mean, std)])

    mnist_train = MNIST(path_to_data, train=True, transform=transforms, download=False)
    mnist_test = MNIST(path_to_data, train=False, transform=transforms, download=False)

    train_loader = data.DataLoader(mnist_train, batch_size=batch_size, shuffle=True)
    test_loader = data.DataLoader(mnist_test, batch_size=batch_size)

    return train_loader, test_loader


def create_dir_if_doesnt_exist(path_to_dir: Path):
    path = Path(path_to_dir)
    if not path.exists():
        log.warning(f"{path_to_dir} does not exist. Creating...")
        path.mkdir(parents=True, exist_ok=True)


def save_conf(conf: Dict, output_dir: Path) -> Path:
    conf_path = output_dir / "download_conf.json"

    with open(conf_path, "w", encoding="utf-8") as f:
        json.dump(conf, f, ensure_ascii=False, indent=4)

    if not conf_path.exists():
        raise IOError("failed to save the config")

    return conf_path


if __name__ == "__main__":
    logging.basicConfig(level=logging.NOTSET)
    log = logging.getLogger("DOWNLOAD")

    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--path_to_output",
        type=str,
        default="./output/data",
        help="Path to store the data",
    )

    args = parser.parse_args()
    conf = vars(args)

    create_dir_if_doesnt_exist(Path(conf["path_to_output"]).resolve())
    download_mnist(Path(conf["path_to_output"]).resolve(), log=log)

    conf_path = save_conf(conf=conf, output_dir=Path(conf["path_to_output"]).resolve())
    log.info(f"Download config saved in {conf_path}")
